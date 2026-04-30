//! HTTP handlers. Chaque sous-module expose un `router()` qui retourne
//! un `axum::Router<AppState>` à nest sous /api ou directement.

pub mod clips;
pub mod events;
pub mod media;
pub mod users;
pub mod voice;
