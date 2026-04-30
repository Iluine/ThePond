use axum::{
    routing::{delete, get, patch, post},
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

use crate::services::auth;
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

    let storage = Storage::new(cfg.uploads_path.clone());
    storage.ensure_dirs()?;
    tracing::info!(uploads = %cfg.uploads_path.display(), "storage ready");

    let hub = SnapshotHub::spawn(pool.clone());
    tracing::info!("snapshot hub ready");

    // ─── Witness HMAC token ──────────────────────────────────
    let secret = auth::load_or_generate_secret();
    let witness_token = auth::witness_token(&secret);
    tracing::info!(
        token = %witness_token,
        "witness token (give témoins this URL) → /orchestration?token={}",
        witness_token
    );

    let app_state = AppState {
        pool: pool.clone(),
        hub,
        pseudo: Arc::new(cfg.pseudo.clone()),
        storage: storage.clone(),
        witness_token: Arc::new(witness_token),
    };

    let event_name = cfg.theme.event_name.clone();

    let api = Router::new()
        .route("/events", get(routes::events::events_handler))
        .route("/pseudo", get(routes::users::sample_pseudo))
        .route("/users", post(routes::users::create_user))
        .route("/users/:id", patch(routes::users::update_user))
        .route("/media", post(routes::media::upload_photo))
        .route("/clips", post(routes::clips::upload_clip))
        .route("/voice", post(routes::voice::upload_voice))
        // Orchestration : protégé par WitnessAuth extractor
        .route("/orchestration/state", get(routes::orchestration::state_handler))
        .route("/orchestration/phases", post(routes::orchestration::create_phase))
        .route("/orchestration/phases/reorder", post(routes::orchestration::reorder_phases))
        .route(
            "/orchestration/phases/:id",
            patch(routes::orchestration::update_phase).delete(routes::orchestration::delete_phase),
        )
        .route(
            "/orchestration/phases/:id/trigger",
            post(routes::orchestration::trigger_phase),
        )
        .route("/orchestration/export", get(routes::orchestration::export_handler))
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
        .nest_service("/uploads", ServeDir::new(&cfg.uploads_path));

    let addr: SocketAddr = cfg.bind_address.parse()?;
    tracing::info!(%addr, "the pond is listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    drop(pool);
    Ok(())
}
