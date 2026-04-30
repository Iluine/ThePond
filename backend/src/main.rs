use axum::{
    routing::{get, patch, post},
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::services::ServeDir;

mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;
mod state;

use crate::services::broadcast::SnapshotHub;
use crate::services::storage::Storage;
use crate::state::AppState;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    version: &'static str,
    event_name: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info,sqlx=warn")),
        )
        .init();

    let cfg = config::Config::load()?;
    tracing::info!(theme = %cfg.theme_path.display(), event = %cfg.theme.event_name, "loaded theme");

    let pool = db::connect(&cfg.db_path).await?;
    db::run_migrations(&pool).await?;
    tracing::info!(db = %cfg.db_path.display(), "database ready");

    // Storage : crée les sous-dossiers et le placeholder clip si absents.
    let storage = Storage::new(cfg.uploads_path.clone());
    storage.ensure_dirs()?;
    tracing::info!(uploads = %cfg.uploads_path.display(), "storage ready");

    // Hub SSE — spawn la tâche debounce + broadcast.
    let hub = SnapshotHub::spawn(pool.clone());
    tracing::info!("snapshot hub ready");

    let app_state = AppState {
        pool: pool.clone(),
        hub,
        pseudo: Arc::new(cfg.pseudo.clone()),
        storage: storage.clone(),
    };

    let event_name = cfg.theme.event_name.clone();

    // Sous-router /api/* — toutes les routes d'API sont sous celui-ci.
    let api = Router::new()
        .route("/events", get(routes::events::events_handler))
        .route("/pseudo", get(routes::users::sample_pseudo))
        .route("/users", post(routes::users::create_user))
        .route("/users/:id", patch(routes::users::update_user))
        .route("/media", post(routes::media::upload_photo))
        .route("/clips", post(routes::clips::upload_clip))
        .route("/voice", post(routes::voice::upload_voice))
        .with_state(app_state);

    let app = Router::new()
        .route(
            "/health",
            get(move || {
                let event_name = event_name.clone();
                async move {
                    Json(Health {
                        status: "ok",
                        version: env!("CARGO_PKG_VERSION"),
                        event_name,
                    })
                }
            }),
        )
        .nest("/api", api)
        // Sert les fichiers uploadés (originaux + thumbnails).
        // En prod, Caddy peut servir directement depuis le volume si
        // on veut court-circuiter le backend pour les statiques.
        .nest_service("/uploads", ServeDir::new(&cfg.uploads_path));

    let addr: SocketAddr = cfg.bind_address.parse()?;
    tracing::info!(%addr, "the pond is listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    drop(pool);
    Ok(())
}
