//! Routes /api/orchestration/* — toutes protégées par WitnessAuth.
//!
//! Liste :
//!   - GET    /state                       → orchestration state (snapshot + dump phases avec triggered_at)
//!   - POST   /phases                      → créer un palier
//!   - PATCH  /phases/:id                  → éditer name et/ou target_time
//!   - DELETE /phases/:id                  → retirer un palier
//!   - POST   /phases/:id/trigger          → forcer le déclenchement (target_time = NOW)
//!   - POST   /phases/reorder              → [{id, phase_order}] pour réordonner en lot
//!   - GET    /export                      → ZIP complet de la mare
//!
//! Toutes les mutations appellent state.hub.mark_dirty() pour que les
//! clients SSE reçoivent un nouveau snapshot après debounce.

use axum::{
    extract::{Path, State},
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::Phase;
use crate::services::auth::WitnessAuth;
use crate::services::export;
use crate::state::AppState;

const EVENT_ID: i64 = 1;

#[derive(Serialize)]
pub struct OrchestrationState {
    pub phases: Vec<Phase>,
    pub counts: OrchestrationCounts,
}

#[derive(Serialize)]
pub struct OrchestrationCounts {
    pub total_users: i64,
    pub total_posts: i64,
    pub posts_visible: i64,
    pub posts_pending: i64,
}

pub async fn state_handler(
    _: WitnessAuth,
    State(state): State<AppState>,
) -> Result<Json<OrchestrationState>, AppError> {
    let phases: Vec<Phase> = sqlx::query_as::<_, Phase>(
        r#"
        SELECT id, phase_order, name, target_time, triggered_at, is_final_reveal
        FROM phases
        WHERE event_id = ?
        ORDER BY phase_order ASC
        "#,
    )
    .bind(EVENT_ID)
    .fetch_all(&state.pool)
    .await?;

    let snap = state.hub.compute_snapshot().await?;

    Ok(Json(OrchestrationState {
        phases,
        counts: OrchestrationCounts {
            total_users: snap.counts.total_users,
            total_posts: snap.counts.total_posts,
            posts_visible: snap.counts.posts_visible,
            posts_pending: snap.counts.posts_pending,
        },
    }))
}

#[derive(Deserialize)]
pub struct CreatePhaseBody {
    pub name: String,
    pub target_time: String,
    #[serde(default)]
    pub is_final_reveal: bool,
}

pub async fn create_phase(
    _: WitnessAuth,
    State(state): State<AppState>,
    Json(body): Json<CreatePhaseBody>,
) -> Result<Json<Phase>, AppError> {
    if body.name.trim().is_empty() || body.name.len() > 80 {
        return Err(AppError::BadRequest("name must be 1..=80 chars".into()));
    }
    if body.target_time.is_empty() || body.target_time.len() > 64 {
        return Err(AppError::BadRequest(
            "target_time must be a non-empty ISO 8601 string".into(),
        ));
    }

    // Place le nouveau palier en fin
    let next_order: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(phase_order) + 1, 0) FROM phases WHERE event_id = ?",
    )
    .bind(EVENT_ID)
    .fetch_one(&state.pool)
    .await?;

    let id: i64 = sqlx::query_scalar(
        r#"
        INSERT INTO phases (event_id, phase_order, name, target_time, is_final_reveal)
        VALUES (?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(EVENT_ID)
    .bind(next_order)
    .bind(body.name.trim())
    .bind(&body.target_time)
    .bind(body.is_final_reveal)
    .fetch_one(&state.pool)
    .await?;

    state.hub.mark_dirty();

    let row: Phase = sqlx::query_as::<_, Phase>(
        "SELECT id, phase_order, name, target_time, triggered_at, is_final_reveal FROM phases WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct UpdatePhaseBody {
    pub name: Option<String>,
    pub target_time: Option<String>,
    pub is_final_reveal: Option<bool>,
}

pub async fn update_phase(
    _: WitnessAuth,
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(body): Json<UpdatePhaseBody>,
) -> Result<Json<Phase>, AppError> {
    let mut name_value: Option<String> = None;
    if let Some(n) = body.name {
        let trimmed = n.trim().to_string();
        if trimmed.is_empty() || trimmed.len() > 80 {
            return Err(AppError::BadRequest("name must be 1..=80 chars".into()));
        }
        name_value = Some(trimmed);
    }

    if let Some(ref t) = body.target_time {
        if t.is_empty() || t.len() > 64 {
            return Err(AppError::BadRequest("target_time empty or too long".into()));
        }
    }

    let result = sqlx::query(
        r#"
        UPDATE phases SET
            name = COALESCE(?, name),
            target_time = COALESCE(?, target_time),
            is_final_reveal = COALESCE(?, is_final_reveal)
        WHERE id = ? AND event_id = ?
        "#,
    )
    .bind(name_value)
    .bind(body.target_time)
    .bind(body.is_final_reveal)
    .bind(id)
    .bind(EVENT_ID)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    state.hub.mark_dirty();

    let row: Phase = sqlx::query_as::<_, Phase>(
        "SELECT id, phase_order, name, target_time, triggered_at, is_final_reveal FROM phases WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_phase(
    _: WitnessAuth,
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result = sqlx::query("DELETE FROM phases WHERE id = ? AND event_id = ?")
        .bind(id)
        .bind(EVENT_ID)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    state.hub.mark_dirty();
    Ok(Json(serde_json::json!({ "ok": true })))
}

/// Force le déclenchement d'un palier en avance : target_time = NOW.
/// PROJECT.md § "Forçage manuel d'un palier" : on ne stocke pas un
/// champ `forced_at` séparé, on déplace simplement target_time. Le
/// champ `triggered_at` est aussi mis à jour pour traçabilité.
pub async fn trigger_phase(
    _: WitnessAuth,
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Phase>, AppError> {
    let now = Utc::now().to_rfc3339();
    let result = sqlx::query(
        "UPDATE phases SET target_time = ?, triggered_at = ? WHERE id = ? AND event_id = ?",
    )
    .bind(&now)
    .bind(&now)
    .bind(id)
    .bind(EVENT_ID)
    .execute(&state.pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }
    state.hub.mark_dirty();

    let row: Phase = sqlx::query_as::<_, Phase>(
        "SELECT id, phase_order, name, target_time, triggered_at, is_final_reveal FROM phases WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(row))
}

#[derive(Deserialize)]
pub struct ReorderItem {
    pub id: i64,
    pub phase_order: i64,
}

pub async fn reorder_phases(
    _: WitnessAuth,
    State(state): State<AppState>,
    Json(items): Json<Vec<ReorderItem>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut tx = state.pool.begin().await?;
    // Étape 1 : on shift TOUS les phase_order de +10000 pour libérer
    // l'espace (sinon UNIQUE(event_id, phase_order) bloque les swaps).
    sqlx::query(
        "UPDATE phases SET phase_order = phase_order + 10000 WHERE event_id = ?",
    )
    .bind(EVENT_ID)
    .execute(&mut *tx)
    .await?;
    // Étape 2 : on applique les nouveaux phase_order pour les ids fournis.
    for item in items {
        sqlx::query("UPDATE phases SET phase_order = ? WHERE id = ? AND event_id = ?")
            .bind(item.phase_order)
            .bind(item.id)
            .bind(EVENT_ID)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    state.hub.mark_dirty();
    Ok(Json(serde_json::json!({ "ok": true })))
}

pub async fn export_handler(
    _: WitnessAuth,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    let bytes = export::build_zip(&state.pool, &state.storage)
        .await
        .map_err(AppError::Other)?;
    let filename = export::filename_for_now();

    let mut response = (axum::http::StatusCode::OK, bytes).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/zip"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"{filename}\""))
            .map_err(|e| AppError::Other(anyhow::anyhow!("header: {e}")))?,
    );
    Ok(response)
}
