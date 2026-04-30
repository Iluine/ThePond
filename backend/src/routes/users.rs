//! POST /api/users  — création d'un canard (génération du pseudo serveur).
//! PATCH /api/users/:id — mise à jour du custom_name.
//!
//! Auth invité : pas d'auth, l'invité fournit son UUID v4 généré côté
//! client au scan QR (PROJECT.md § "Authentification"). Le UUID sert
//! d'identifiant pour tous les uploads.

use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use serde::Deserialize;

use crate::error::AppError;
use crate::models::Canard;
use crate::services::pseudo;
use crate::state::AppState;

const ALLOWED_COLORS: &[&str] = &["yellow", "white", "blue", "rainbow"];

#[derive(Debug, Deserialize)]
pub struct CreateUserBody {
    pub id: String,
    pub duck_color: String,
    pub custom_name: Option<String>,
    /// Pseudo choisi par le client (issu d'un GET /api/pseudo précédent).
    /// Si absent ou invalide, le serveur en génère un.
    pub pseudo: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PseudoResponse {
    pub pseudo: String,
}

/// GET /api/pseudo — génère un pseudo sans persister. Utilisé par
/// WelcomeView pour le bouton "Reroule" : chaque clic = un nouvel
/// échantillonnage. Sur PLONGER, le client renvoie le dernier pseudo
/// affiché dans POST /api/users.
pub async fn sample_pseudo(State(state): State<AppState>) -> Json<PseudoResponse> {
    Json(PseudoResponse {
        pseudo: pseudo::generate(&state.pseudo),
    })
}

pub async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserBody>,
) -> Result<Json<Canard>, AppError> {
    // ─── Validation ─────────────────────────────────────────
    if body.id.is_empty() || body.id.len() > 64 {
        return Err(AppError::BadRequest("id must be 1..=64 chars".into()));
    }
    if !ALLOWED_COLORS.contains(&body.duck_color.as_str()) {
        return Err(AppError::BadRequest(format!(
            "duck_color must be one of {ALLOWED_COLORS:?}"
        )));
    }
    let custom_name = body.custom_name.as_ref().map(|s| s.trim().to_string());
    if let Some(name) = &custom_name {
        if name.len() > 80 {
            return Err(AppError::BadRequest("custom_name max 80 chars".into()));
        }
    }

    // ─── Pseudo : utilise celui du client si valide, sinon génère ──
    let pseudo = match body.pseudo.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(p) if p.len() <= 100 => p.to_string(),
        Some(_) => return Err(AppError::BadRequest("pseudo too long (max 100)".into())),
        None => pseudo::generate(&state.pseudo),
    };
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO users (id, pseudo, custom_name, duck_color, created_at, last_seen_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&body.id)
    .bind(&pseudo)
    .bind(custom_name.as_deref().filter(|s| !s.is_empty()))
    .bind(&body.duck_color)
    .bind(&now)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    // ─── Notifie le hub : counts.total_users a changé ────────
    state.hub.mark_dirty();

    // ─── Recharge depuis la DB pour renvoyer le shape exact ─
    let canard: Canard = sqlx::query_as::<_, Canard>(
        r#"
        SELECT id, pseudo, custom_name, duck_color, created_at, last_seen_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(&body.id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(canard))
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserBody {
    /// null = retirer le custom_name, sinon on stocke trim().
    pub custom_name: Option<String>,
}

pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateUserBody>,
) -> Result<Json<Canard>, AppError> {
    let custom_name = body.custom_name.map(|s| s.trim().to_string());
    if let Some(name) = &custom_name {
        if name.len() > 80 {
            return Err(AppError::BadRequest("custom_name max 80 chars".into()));
        }
    }
    let custom_name = custom_name.filter(|s| !s.is_empty());

    let result = sqlx::query("UPDATE users SET custom_name = ? WHERE id = ?")
        .bind(custom_name.as_deref())
        .bind(&id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    state.hub.mark_dirty();

    let canard: Canard = sqlx::query_as::<_, Canard>(
        r#"
        SELECT id, pseudo, custom_name, duck_color, created_at, last_seen_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(canard))
}
