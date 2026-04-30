use serde::Serialize;

/// Un palier (Apéro, Dîner, Dessert, Café, Le réveil de la mare, etc.).
///
/// `target_time` est un timestamp ISO 8601 ; le calcul current/visible
/// se fait via la comparaison textuelle (les ISO timestamps sont
/// lexicographiquement ordonnables tant qu'ils utilisent le même format
/// — on forcera l'UTC ou +HH:MM partout).
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Phase {
    pub id: i64,
    pub phase_order: i64,
    pub name: String,
    pub target_time: String,
    pub triggered_at: Option<String>,
    pub is_final_reveal: bool,
}
