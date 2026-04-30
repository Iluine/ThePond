//! POST /api/media — upload d'une photo.
//!
//! Multipart : champ `user_id` (texte) + champ `file` (binaire).
//! Le serveur sauvegarde l'original tel quel, génère un thumbnail
//! 600×600 max en JPEG q80, insère en DB et notifie le hub.
//!
//! HEIC : non supporté en V1 (libheif est lourd à packager).
//! Si l'invité poste un .heic, le client doit avoir converti en JPEG
//! avant l'upload — cette responsabilité tombera sur le frontend au
//! prompt 8 (UploadView + capture in-app).

use std::path::Path;

use axum::{
    extract::{Multipart, State},
    Json,
};
use axum::body::Bytes;
use chrono::Utc;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::Media;
use crate::services::storage::Kind;
use crate::state::AppState;

/// 30 Mo max — confortable pour des JPEG modernes 12-24 Mp.
const MAX_PHOTO_BYTES: usize = 30 * 1024 * 1024;
const THUMB_MAX_DIM: u32 = 600;
const THUMB_QUALITY: u8 = 80;

pub async fn upload_photo(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Media>, AppError> {
    let mut user_id: Option<String> = None;
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
            "file" => {
                content_type = field.content_type().map(String::from);
                filename_hint = field.file_name().map(String::from);
                let bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(format!("file: {e}")))?;
                if bytes.len() > MAX_PHOTO_BYTES {
                    return Err(AppError::BadRequest(format!(
                        "file too large ({} bytes > {} max)",
                        bytes.len(),
                        MAX_PHOTO_BYTES
                    )));
                }
                file_bytes = Some(bytes);
            }
            _ => {}
        }
    }

    let user_id = user_id.ok_or_else(|| AppError::BadRequest("user_id field required".into()))?;
    let bytes = file_bytes.ok_or_else(|| AppError::BadRequest("file field required".into()))?;

    // Vérifie que le canard existe
    let user_exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM users WHERE id = ?")
        .bind(&user_id)
        .fetch_optional(&state.pool)
        .await?;
    if user_exists.is_none() {
        return Err(AppError::Unauthorized);
    }

    let ext = pick_photo_extension(content_type.as_deref(), filename_hint.as_deref());

    let (filename, original_path) = state.storage.allocate(Kind::PhotoOriginal, ext);
    let (thumb_filename, thumb_path) = state.storage.thumb_for(Kind::PhotoOriginal, &filename);

    // Sauvegarde de l'original (async)
    tokio::fs::write(&original_path, &bytes)
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("save original: {e}")))?;

    // Thumbnail — image crate est sync, donc spawn_blocking
    let bytes_for_thumb = bytes.clone();
    let thumb_path_owned = thumb_path.clone();
    tokio::task::spawn_blocking(move || generate_jpeg_thumb(&bytes_for_thumb, &thumb_path_owned))
        .await
        .map_err(|e| AppError::Other(anyhow::anyhow!("spawn_blocking join: {e}")))?
        .map_err(|e| AppError::Other(anyhow::anyhow!("thumbnail: {e}")))?;

    // INSERT
    let media_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    sqlx::query(
        r#"
        INSERT INTO media (id, user_id, filename, thumb_filename, posted_at, hidden)
        VALUES (?, ?, ?, ?, ?, 0)
        "#,
    )
    .bind(&media_id)
    .bind(&user_id)
    .bind(&filename)
    .bind(&thumb_filename)
    .bind(&now)
    .execute(&state.pool)
    .await?;

    state.hub.mark_dirty();

    let media: Media = sqlx::query_as::<_, Media>(
        r#"
        SELECT id, user_id, filename, thumb_filename, posted_at, hidden
        FROM media
        WHERE id = ?
        "#,
    )
    .bind(&media_id)
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(media))
}

fn pick_photo_extension(content_type: Option<&str>, filename: Option<&str>) -> &'static str {
    if let Some(ct) = content_type {
        match ct {
            "image/jpeg" => return "jpg",
            "image/png" => return "png",
            _ => {}
        }
    }
    if let Some(name) = filename {
        let ext = Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_ascii_lowercase());
        match ext.as_deref() {
            Some("jpg") | Some("jpeg") => return "jpg",
            Some("png") => return "png",
            _ => {}
        }
    }
    "jpg"
}

fn generate_jpeg_thumb(input: &[u8], output_path: &Path) -> anyhow::Result<()> {
    use image::codecs::jpeg::JpegEncoder;
    use std::fs::File;
    use std::io::BufWriter;

    let img = image::load_from_memory(input)?;
    let thumb = img.thumbnail(THUMB_MAX_DIM, THUMB_MAX_DIM);

    let f = File::create(output_path)?;
    let mut writer = BufWriter::new(f);
    let encoder = JpegEncoder::new_with_quality(&mut writer, THUMB_QUALITY);
    thumb.write_with_encoder(encoder)?;
    Ok(())
}
