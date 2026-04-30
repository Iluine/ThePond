//! Export ZIP de toute la mare.
//!
//! Contenu du zip :
//!   - data.json : dump des tables users, phases, media, clips,
//!     voice_messages, likes (un object racine avec ces 6 clés)
//!   - uploads/... : tous les fichiers du UPLOADS_PATH, à plat
//!     dans la même hiérarchie qu'on avait sur disque
//!
//! Génération in-memory pour V1. Si la mare devient grosse (>500 MB)
//! on switchera sur du streaming.

use std::io::{Cursor, Write};
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::SqlitePool;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use crate::services::storage::Storage;

#[derive(Serialize)]
struct Dump {
    exported_at: String,
    users: Vec<Value>,
    phases: Vec<Value>,
    media: Vec<Value>,
    clips: Vec<Value>,
    voice_messages: Vec<Value>,
    likes: Vec<Value>,
}

pub async fn build_zip(pool: &SqlitePool, storage: &Storage) -> Result<Vec<u8>> {
    // ─── Dump SQL → JSON ─────────────────────────────────────
    let dump = Dump {
        exported_at: chrono::Utc::now().to_rfc3339(),
        users: rows_as_json(pool, "SELECT * FROM users").await?,
        phases: rows_as_json(pool, "SELECT * FROM phases").await?,
        media: rows_as_json(pool, "SELECT * FROM media").await?,
        clips: rows_as_json(pool, "SELECT * FROM clips").await?,
        voice_messages: rows_as_json(pool, "SELECT * FROM voice_messages").await?,
        likes: rows_as_json(pool, "SELECT * FROM likes").await?,
    };
    let dump_json = serde_json::to_vec_pretty(&dump)?;

    // ─── Construction du zip ─────────────────────────────────
    let buf: Vec<u8> = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(buf));
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    zip.start_file("data.json", opts)
        .context("zip start_file data.json")?;
    zip.write_all(&dump_json)
        .context("zip write data.json")?;

    // Walk uploads/ et ajoute chaque fichier dans uploads/...
    add_uploads_recursive(&mut zip, storage.root(), Path::new("uploads"))?;

    let cursor = zip.finish().context("zip finish")?;
    Ok(cursor.into_inner())
}

async fn rows_as_json(pool: &SqlitePool, sql: &str) -> Result<Vec<Value>> {
    use sqlx::{Column, Row, TypeInfo};

    let rows = sqlx::query(sql).fetch_all(pool).await?;
    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut obj = serde_json::Map::new();
        for col in row.columns() {
            let name = col.name().to_string();
            // Try to get values in a few common types ; SQLite is dynamic
            // so we sniff via the column's declared type.
            let value: Value = match col.type_info().name() {
                "TEXT" => row
                    .try_get::<Option<String>, _>(name.as_str())
                    .ok()
                    .flatten()
                    .map(Value::String)
                    .unwrap_or(Value::Null),
                "INTEGER" | "INT" => row
                    .try_get::<Option<i64>, _>(name.as_str())
                    .ok()
                    .flatten()
                    .map(|n| Value::Number(n.into()))
                    .unwrap_or(Value::Null),
                "REAL" | "FLOAT" => row
                    .try_get::<Option<f64>, _>(name.as_str())
                    .ok()
                    .flatten()
                    .and_then(|f| serde_json::Number::from_f64(f).map(Value::Number))
                    .unwrap_or(Value::Null),
                _ => Value::Null,
            };
            obj.insert(name, value);
        }
        out.push(Value::Object(obj));
    }
    Ok(out)
}

fn add_uploads_recursive<W: Write + std::io::Seek>(
    zip: &mut ZipWriter<W>,
    base: &Path,
    relative_dir: &Path,
) -> Result<()> {
    let dir = base.join(
        relative_dir
            .strip_prefix("uploads")
            .unwrap_or(relative_dir),
    );
    if !dir.exists() {
        return Ok(());
    }
    let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("read_dir {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;
        let name = entry.file_name();
        let in_zip = relative_dir.join(&name);
        if meta.is_dir() {
            add_uploads_recursive(zip, base, &in_zip)?;
        } else if meta.is_file() {
            let zip_path = in_zip
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("non-utf8 path in uploads"))?
                .to_string();
            zip.start_file(zip_path, opts)
                .with_context(|| format!("zip start_file {}", path.display()))?;
            let bytes = std::fs::read(&path)
                .with_context(|| format!("read {}", path.display()))?;
            zip.write_all(&bytes)
                .with_context(|| format!("zip write {}", path.display()))?;
        }
    }
    Ok(())
}

#[derive(Serialize)]
pub struct ExportFilename {
    pub name: String,
}

pub fn filename_for_now() -> String {
    let now = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    format!("thepond-{now}.zip")
}
