use serde::Serialize;

/// Un média polymorphe : photo, clip ou voice. Le champ `kind`
/// discrimine le type, et les autres champs sont peuplés selon le kind :
///   - photo : pas de duration / waveform / caption
///   - clip  : duration_seconds, pas de waveform / caption
///   - voice : duration_seconds + waveform_json + caption optionnel
///
/// `user_pseudo` et `user_color` sont joints depuis la table users côté
/// backend pour éviter au client un round-trip par carte.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Media {
    pub id: String,
    pub kind: String,
    pub user_id: String,
    pub filename: String,
    pub thumb_filename: String,
    pub posted_at: String,
    pub hidden: bool,
    pub duration_seconds: Option<f64>,
    pub waveform_json: Option<String>,
    pub caption: Option<String>,
    pub user_pseudo: Option<String>,
    pub user_color: Option<String>,
}
