//! Calcul du snapshot complet à partir de l'état DB.
//!
//! `media_visible` et `media_recent_for_tv` unionent les trois tables
//! (media, clips, voice_messages) pour exposer un flux Media polymorphe
//! au frontend. Le champ `kind` discrimine le type côté UI.
//!
//! Le LEFT JOIN sur users évite au client un round-trip par carte pour
//! récupérer le pseudo et la couleur du canard.

use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;

use crate::models::{Media, Phase, Snapshot, SnapshotCounts};
use crate::services::phase_logic;

const EVENT_ID: i64 = 1;

/// SELECT polymorphe sur les 3 tables, fenêtre temporelle bornée.
const SQL_WINDOW_BOTH: &str = r#"
SELECT id, 'photo' AS kind, user_id, filename, thumb_filename, posted_at, hidden,
       NULL AS duration_seconds, NULL AS waveform_json, NULL AS caption,
       (SELECT pseudo FROM users WHERE id = m.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = m.user_id) AS user_color
FROM media m
WHERE posted_at >= ?1 AND posted_at < ?2 AND hidden = 0
UNION ALL
SELECT id, 'clip' AS kind, user_id, filename, thumb_filename, posted_at, hidden,
       duration_seconds, NULL AS waveform_json, NULL AS caption,
       (SELECT pseudo FROM users WHERE id = c.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = c.user_id) AS user_color
FROM clips c
WHERE posted_at >= ?1 AND posted_at < ?2 AND hidden = 0
UNION ALL
SELECT id, 'voice' AS kind, user_id, filename, '' AS thumb_filename, posted_at, hidden,
       duration_seconds, waveform_json, caption,
       (SELECT pseudo FROM users WHERE id = v.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = v.user_id) AS user_color
FROM voice_messages v
WHERE posted_at >= ?1 AND posted_at < ?2 AND hidden = 0
ORDER BY posted_at DESC
"#;

/// SELECT polymorphe sur les 3 tables, fenêtre ouverte (>= start, sans
/// upper bound). Utilisé pour media_recent_for_tv et state 4 (révélation).
const SQL_WINDOW_OPEN: &str = r#"
SELECT id, 'photo' AS kind, user_id, filename, thumb_filename, posted_at, hidden,
       NULL AS duration_seconds, NULL AS waveform_json, NULL AS caption,
       (SELECT pseudo FROM users WHERE id = m.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = m.user_id) AS user_color
FROM media m
WHERE posted_at >= ?1 AND hidden = 0
UNION ALL
SELECT id, 'clip' AS kind, user_id, filename, thumb_filename, posted_at, hidden,
       duration_seconds, NULL AS waveform_json, NULL AS caption,
       (SELECT pseudo FROM users WHERE id = c.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = c.user_id) AS user_color
FROM clips c
WHERE posted_at >= ?1 AND hidden = 0
UNION ALL
SELECT id, 'voice' AS kind, user_id, filename, '' AS thumb_filename, posted_at, hidden,
       duration_seconds, waveform_json, caption,
       (SELECT pseudo FROM users WHERE id = v.user_id) AS user_pseudo,
       (SELECT duck_color FROM users WHERE id = v.user_id) AS user_color
FROM voice_messages v
WHERE posted_at >= ?1 AND hidden = 0
ORDER BY posted_at DESC
"#;

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

    // Médias visibles dans la galerie publique :
    //   - état 4 (final reveal) → tous les médias depuis le début de
    //     l'événement (posted_at >= phase_visible.target_time si défini,
    //     sinon depuis l'epoch — concrètement le premier palier)
    //   - autres états → fenêtre [visible.target_time, current.target_time)
    let media_visible: Vec<Media> = if let Some(cur) = &phase_current {
        if cur.is_final_reveal {
            // Tout dévoiler : depuis le début (premier palier)
            let start = phases_all
                .first()
                .map(|p| p.target_time.as_str())
                .unwrap_or("");
            sqlx::query_as::<_, Media>(SQL_WINDOW_OPEN)
                .bind(start)
                .fetch_all(pool)
                .await?
        } else if let Some(vis) = &phase_visible {
            sqlx::query_as::<_, Media>(SQL_WINDOW_BOTH)
                .bind(&vis.target_time)
                .bind(&cur.target_time)
                .fetch_all(pool)
                .await?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // Médias récents pour Mare TV : palier en cours, pas encore visibles.
    let media_recent_for_tv: Vec<Media> = match phase_current.as_ref() {
        Some(current) => sqlx::query_as::<_, Media>(SQL_WINDOW_OPEN)
            .bind(&current.target_time)
            .fetch_all(pool)
            .await?,
        None => Vec::new(),
    };

    let counts = compute_counts(
        pool,
        media_visible.len() as i64,
        media_recent_for_tv.len() as i64,
    )
    .await?;

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

    // total_posts somme les 3 tables (photos + clips + vocaux non hidden).
    let total_posts: i64 = sqlx::query_scalar(
        r#"
        SELECT
            (SELECT COUNT(*) FROM media WHERE hidden = 0)
          + (SELECT COUNT(*) FROM clips WHERE hidden = 0)
          + (SELECT COUNT(*) FROM voice_messages WHERE hidden = 0)
        "#,
    )
    .fetch_one(pool)
    .await?;

    Ok(SnapshotCounts {
        total_users,
        total_posts,
        posts_visible,
        posts_pending,
    })
}
