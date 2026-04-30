//! État partagé Axum, propagé via `State<AppState>` aux handlers.

use std::sync::Arc;

use sqlx::SqlitePool;

use crate::config::PseudoConfig;
use crate::services::broadcast::SnapshotHub;
use crate::services::storage::Storage;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub hub: Arc<SnapshotHub>,
    pub pseudo: Arc<PseudoConfig>,
    pub storage: Storage,
    /// Token public dérivé de WITNESS_SECRET, comparé contre ?token=
    /// par l'extractor WitnessAuth.
    pub witness_token: Arc<String>,
}
