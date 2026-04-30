//! Logique métier non-routée : calcul du snapshot, broadcast SSE,
//! services partagés. Les routes (handlers HTTP) appellent ces services.

pub mod broadcast;
pub mod phase_logic;
pub mod snapshot;
