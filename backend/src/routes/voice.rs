//! POST /api/voice — upload d'un message vocal (60s max attendu côté client).
//!
//! V1 limitation : pas de calcul de waveform via symphonia. On stocke
//! l'original et un waveform_json placeholder (100 valeurs uniformes 0.5).
//! Le frontend peut quand même afficher un waveform "plat" correct ; on
//! remplacera par les vraies amplitudes échantillonnées dans un prompt
//! dédié quand on attaquera la pipeline audio complète.

use std::path::Path;

use axum::{
    extract::{Multipart, State},
    Json,
};
use axum::body::Bytes;
use chrono::Utc;
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppError;
use crate::services::storage::Kind;
use crate::state::AppState;

/// 10 Mo max — un vocal 60s en webm/opus tourne autour de 600 ko, on a
/// largement de la marge pour un AAC 64 kbps × 60s = 480 ko.
const MAX_VOICE_BYTES: usize = 10 * 1024 * 1024;
const PLACEHOLDER_WAVEFORM_BARS: usize = 100;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct VoiceMessage {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub waveform_json: String,
    pub duration_seconds: f64,
    pub caption: Option<String>,
    pub posted_at: String,
    pub hidden: bool,
}

pub async fn upload_voice(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<VoiceMessage>, AppError> {
    let mut user_id: Option<String> = None;
    let mut duration_seconds: Option<f64> = None;
    let mut caption: Option<String> = None;
    let mut file_bytes: Option<Bytes> = None;
    let mut content_type: Option<String> = None;
    let mut filename_hint: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("multipart error: {e}")))?
    {
        match field.name().unwrap_or("") {
            "user_id" => {
                user_id = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::BadRequest(format!("user_id: {e}")))?,
                );
            }
            "duration_seconds" => {
                let txt = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("duration_seconds: {e}")))?;
                duration_seconds = Some(txt.trim().parse().map_err(|e| {
                    AppError::BadRequest(format!("duration_seconds not a number: {e}"))
                })?);
            }
            "caption" => {
                let txt = field
                    .text()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("caption: {e}")))?;
                let trimmed = txt.trim();
                if !trimmed.is_empty() {
                    if trimmed.len() > 280 {
                        return Err(AppError::BadRequest("caption max 280 chars".into()));
                    }
                    caption = Some(trimmed.to_string());
                }
            }
            "file" => {
                content_type = field.content_type().map(String::from);
                filename_hint = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("file: {e}")))?;
                if bytes.len() > MAX_VOICE_BYTES {
                    return Err(AppError::BadRequest(format!(
                        "file too large ({} bytes > {} max)",
                        bytes.len(),
                        MAX_VOICE_BYTES
                    )));
                }
                file_bytes = Some(bytes);
            }
            _ => {}
        }
    }

    let user_id = user_id.ok_or_else(|| AppError::BadRequest("user_id field required".into()))?;
    let duration_seconds =
        duration_seconds.ok_or_else(|| AppError::BadRequest("duration_seconds field required".into()))?;
    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err(AppError::BadRequest("duration_seconds must be > 0".into()));
    }
    let bytes = file_bytes.ok_or_else(|| AppError::BadRequest("file field required".into()))?;

    let user_exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await?;
    if user_exists.is_none() {
        return Err(AppError::Unauthorized);
    }

    let ext = pick_voice_extension(content_type.as_deref(), filename_hint.as_deref());
    let (filename, original_path) = state.storage.allocate(Kind::Voice, ext);

    tokio::fs::write(&original_path, &bytes)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("save voice: {e}")))?;

    // Waveform placeholder : 100 valeurs uniformes (le client affiche
    // un waveform "plat"). Remplacé par les vraies amplitudes quand
    // la pipeline symphonia sera branchée.
    let waveform_json = serde_json::to_string(&vec![0.5_f32; PLACEHOLDER_WAVEFORM_BARS])
        .expect("serializing fixed-size f32 array always succeeds");

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO voice_messages
            (id, user_id, filename, waveform_json, duration_seconds, caption, posted_at, hidden)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0)
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&filename)
    .bind(&waveform_json)
    .bind(duration_seconds)
    .bind(caption.as_deref())
    .bind(&now)
    .execute(&state.pool)
    .await?;

    state.hub.mark_dirty();

    let voice: VoiceMessage = sqlx::query_as::<_, VoiceMessage>(
        r#"
        SELECT id, user_id, filename, waveform_json, duration_seconds, caption, posted_at, hidden
        FROM voice_messages
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(voice))
}

fn pick_voice_extension(content_type: Option<&str>, filename: Option<&str>) -> &'static str {
    if let Some(ct) = content_type {
        match ct {
            "audio/webm" => return "webm",
            "audio/mp4" => return "m4a",
            "audio/mpeg" => return "mp3",
            "audio/ogg" => return "ogg",
            _ => {}
        }
    }
    if let Some(name) = filename {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("webm") => return "webm",
            Some("m4a") | Some("mp4") => return "m4a",
            Some("mp3") => return "mp3",
            Some("ogg") => return "ogg",
            _ => {}
        }
    }
    "webm"
}
