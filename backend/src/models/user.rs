use serde::Serialize;

/// Un canard (utilisateur invité). Mappe sur la table `users`.
/// Le champ `duck_color` est contraint en DB par
/// `CHECK (duck_color IN ('yellow', 'white', 'blue', 'rainbow'))`.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Canard {
    pub id: String,
    pub pseudo: String,
    pub custom_name: Option<String>,
    pub duck_color: String,
    pub created_at: String,
    pub last_seen_at: Option<String>,
}
