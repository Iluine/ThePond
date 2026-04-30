use serde::Serialize;

/// Un média photo. Pour V1 prompt 5, le snapshot ne contient que les
/// photos. Les clips et vocaux seront unionés au prompt 6 quand les
/// routes upload arrivent.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Media {
    pub id: String,
    pub user_id: String,
    pub filename: String,
    pub thumb_filename: String,
    pub posted_at: String,
    pub hidden: bool,
}
