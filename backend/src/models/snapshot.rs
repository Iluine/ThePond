use serde::Serialize;

use super::{Media, Phase};

/// Snapshot complet de l'état de la mare. Le client remplace son store
/// local par ce snapshot à chaque changement (pas de delta — voir
/// PROJECT.md § "Architecture temps réel").
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    /// ISO 8601 — heure du serveur, utile pour mesurer le drift client.
    pub server_time: String,
    /// Palier en cours (le dernier dont target_time <= NOW). None si
    /// aucun palier n'a encore démarré.
    pub phase_current: Option<Phase>,
    /// Palier visible dans la galerie (le précédent du current). None
    /// quand on est sur le premier palier ou que phase_current est None.
    pub phase_visible: Option<Phase>,
    /// Tous les paliers (pour orchestration témoins).
    pub phases_all: Vec<Phase>,
    /// Médias visibles dans la galerie (postés dans la fenêtre du
    /// palier visible).
    pub media_visible: Vec<Media>,
    /// Médias récents pour Mare TV (palier en cours, pas encore visibles
    /// dans la galerie publique).
    pub media_recent_for_tv: Vec<Media>,
    pub counts: SnapshotCounts,
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotCounts {
    pub total_users: i64,
    pub total_posts: i64,
    pub posts_visible: i64,
    pub posts_pending: i64,
}
