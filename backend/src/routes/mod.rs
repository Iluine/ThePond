//! HTTP handlers. Chaque sous-module expose un `router()` qui retourne
//! un `axum::Router<AppState>` à nest sous /api ou directement.

pub mod events;
