//! POST /api/clips — upload d'un clip vidéo (15s max attendu côté client).
//!
//! V1 limitation : pas d'extraction de frame ffmpeg. On stocke l'original,
//! et on pointe `thumb_filename` vers un placeholder partagé. La pipeline
//! ffmpeg réelle (frame à 1s, JPEG) viendra dans un prompt dédié quand on
//! attaquera la pipeline média complète. Le schéma DB n'a rien à changer.

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
use crate::services::storage::{Kind, Storage};
use crate::state::AppState;

/// 100 Mo max — clip mobile compressé 1080p typique tient dedans.
const MAX_CLIP_BYTES: usize = 100 * 1024 * 1024;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Clip {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub thumb_filename: String,
    pub duration_seconds: f64,
    pub posted_at: String,
    pub hidden: bool,
}

pub async fn upload_clip(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Clip>, AppError> {
    let mut user_id: Option<String> = None;
    let mut duration_seconds: Option<f64> = None;
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
            "file" => {
                content_type = field.content_type().map(String::from);
                filename_hint = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("file: {e}")))?;
                if bytes.len() > MAX_CLIP_BYTES {
                    return Err(AppError::BadRequest(format!(
                        "file too large ({} bytes > {} max)",
                        bytes.len(),
                        MAX_CLIP_BYTES
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

    // user existe ?
    let user_exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await?;
    if user_exists.is_none() {
        return Err(AppError::Unauthorized);
    }

    let ext = pick_clip_extension(content_type.as_deref(), filename_hint.as_deref());
    let (filename, original_path) = state.storage.allocate(Kind::ClipOriginal, ext);

    tokio::fs::write(&original_path, &bytes)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("save clip: {e}")))?;

    // Placeholder partagé — voir storage::Storage::CLIP_PLACEHOLDER_FILENAME
    let thumb_filename = Storage::CLIP_PLACEHOLDER_FILENAME.to_string();

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO clips (id, user_id, filename, thumb_filename, duration_seconds, posted_at, hidden)
        VALUES (?, ?, ?, ?, ?, ?, 0)
        "#,
    )
    .bind(&id)
    .bind(&user_id)
    .bind(&filename)
    .bind(&thumb_filename)
    .bind(duration_seconds)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    state.hub.mark_dirty();

    let clip: Clip = sqlx::query_as::<_, Clip>(
        r#"
        SELECT id, user_id, filename, thumb_filename, duration_seconds, posted_at, hidden
        FROM clips
        WHERE id = ?
        "#,
    )
    .bind(&id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(clip))
}

fn pick_clip_extension(content_type: Option<&str>, filename: Option<&str>) -> &'static str {
    if let Some(ct) = content_type {
        match ct {
            "video/mp4" => return "mp4",
            "video/webm" => return "webm",
            _ => {}
        }
    }
    if let Some(name) = filename {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("mp4") => return "mp4",
            Some("webm") => return "webm",
            Some("mov") => return "mp4", // remap iOS .mov sur .mp4
            _ => {}
        }
    }
    "mp4"
}
