//! GET /api/events — endpoint Server-Sent Events.
//!
//! À chaque connexion, le serveur envoie immédiatement le snapshot
//! courant. Ensuite, il stream chaque snapshot diffusé par le hub
//! (après debounce 300 ms côté backend). En cas de lag (client trop
//! lent), on resync en recomputant le snapshot courant pour ne jamais
//! laisser le client dans un état périmé silencieusement.

use std::convert::Infallible;
use std::time::Duration;

use async_stream::stream;
use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use futures_core::stream::Stream;
use tokio::sync::broadcast::error::RecvError;

use crate::state::AppState;

pub async fn events_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let hub = state.hub.clone();
    let mut rx = hub.subscribe();
    let initial = hub.compute_snapshot().await;

    let s = stream! {
        // 1. Snapshot initial (immédiatement à la connexion)
        match initial {
            Ok(snap) => match Event::default().json_data(&snap) {
                Ok(ev) => yield Ok(ev),
                Err(err) => tracing::error!(%err, "failed to serialize initial snapshot"),
            },
            Err(err) => tracing::error!(%err, "failed to compute initial snapshot"),
        }

        // 2. Stream des snapshots diffusés
        loop {
            match rx.recv().await {
                Ok(snap) => match Event::default().json_data(&*snap) {
                    Ok(ev) => yield Ok(ev),
                    Err(err) => tracing::error!(%err, "failed to serialize broadcast snapshot"),
                },
                Err(RecvError::Lagged(n)) => {
                    tracing::warn!(lagged_by = n, "SSE client lagged, resyncing");
                    if let Ok(snap) = hub.compute_snapshot().await {
                        if let Ok(ev) = Event::default().json_data(&snap) {
                            yield Ok(ev);
                        }
                    }
                }
                Err(RecvError::Closed) => break,
            }
        }
    };

    Sse::new(s).keep_alive(
        // Heartbeat 15 s — maintient la connexion en vie à travers les
        // proxys / reverse-proxys timides (Caddy, Cloudflare).
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    )
}
