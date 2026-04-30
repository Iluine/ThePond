//! Logique métier non-routée : calcul du snapshot, broadcast SSE,
//! services partagés. Les routes (handlers HTTP) appellent ces services.

pub mod auth;
pub mod broadcast;
pub mod export;
pub mod phase_logic;
pub mod pseudo;
pub mod snapshot;
pub mod storage;
