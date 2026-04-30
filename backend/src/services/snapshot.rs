//! Calcul du snapshot complet à partir de l'état DB.
//!
//! Pour V1 prompt 5, seul le table `media` (photos) est consulté. Les
//! tables `clips` et `voice_messages` seront unionées au prompt 6 quand
//! les routes upload arriveront — c'est juste un OR sur des SELECT
//! supplémentaires, la logique de fenêtre temporelle reste identique.

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{Media, Phase, Snapshot, SnapshotCounts};
use crate::services::phase_logic;

const EVENT_ID: i64 = 1;

pub async fn compute(pool: &SqlitePool) -> Result<Snapshot> {
    let now_iso = Utc::now().to_rfc3339();

    let phases_all: Vec<Phase> = sqlx::query_as::<_, Phase>(
        r#"
        SELECT id, phase_order, name, target_time, triggered_at, is_final_reveal
        FROM phases
        WHERE event_id = ?
        ORDER BY phase_order ASC
        "#,
    )
    .bind(EVENT_ID)
    .fetch_all(pool)
    .await?;

    let phase_current = phase_logic::current_phase(&phases_all, &now_iso);
    let phase_visible = phase_logic::visible_phase(&phases_all, phase_current.as_ref());

    // Médias visibles : postés dans la fenêtre du palier visible
    // [visible.target_time, current.target_time)
    let media_visible: Vec<Media> = match (phase_visible.as_ref(), phase_current.as_ref()) {
        (Some(visible), Some(current)) => {
            sqlx::query_as::<_, Media>(
                r#"
                SELECT id, user_id, filename, thumb_filename, posted_at, hidden
                FROM media
                WHERE posted_at >= ? AND posted_at < ? AND hidden = 0
                ORDER BY posted_at DESC
                "#,
            )
            .bind(&visible.target_time)
            .bind(&current.target_time)
            .fetch_all(pool)
            .await?
        }
        _ => Vec::new(),
    };

    // Médias récents pour Mare TV : postés depuis le début du palier en
    // cours (pas encore visibles dans la galerie publique).
    let media_recent_for_tv: Vec<Media> = match phase_current.as_ref() {
        Some(current) => {
            sqlx::query_as::<_, Media>(
                r#"
                SELECT id, user_id, filename, thumb_filename, posted_at, hidden
                FROM media
                WHERE posted_at >= ? AND hidden = 0
                ORDER BY posted_at DESC
                "#,
            )
            .bind(&current.target_time)
            .fetch_all(pool)
            .await?
        }
        None => Vec::new(),
    };

    let counts = compute_counts(pool, media_visible.len() as i64, media_recent_for_tv.len() as i64).await?;

    Ok(Snapshot {
        server_time: now_iso,
        phase_current,
        phase_visible,
        phases_all,
        media_visible,
        media_recent_for_tv,
        counts,
    })
}

async fn compute_counts(
    pool: &SqlitePool,
    posts_visible: i64,
    posts_pending: i64,
) -> Result<SnapshotCounts> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let total_posts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM media WHERE hidden = 0")
        .fetch_one(pool)
        .await?;

    Ok(SnapshotCounts {
        total_users,
        total_posts,
        posts_visible,
        posts_pending,
    })
}
