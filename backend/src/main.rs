use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::net::SocketAddr;
use std::sync::Arc;

mod config;
mod db;
mod error;
mod models;
mod routes;
mod services;

use crate::services::broadcast::SnapshotHub;

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

    // Hub SSE — spawn la tâche debounce + broadcast.
    let hub = SnapshotHub::spawn(pool.clone());
    tracing::info!("snapshot hub ready");

    let event_name = cfg.theme.event_name.clone();

    // Sous-router pour /api/* — toutes les routes d'API montent ici.
    let api = Router::new()
        .route("/events", get(routes::events::events_handler))
        .with_state(hub);

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
        .nest("/api", api);

    let addr: SocketAddr = cfg.bind_address.parse()?;
    tracing::info!(%addr, "the pond is listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    // Garder ce drop explicit pour bien signaler que le pool ferme proprement
    drop(pool);
    Ok(())
}
