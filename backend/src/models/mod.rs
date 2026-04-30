//! Modèles serde + sqlx miroirs des types TS frontend.
//!
//! Convention de nommage : on garde snake_case identique entre Rust et TS
//! pour que la sérialisation par défaut serde matche directement le shape
//! attendu par useSnapshotStore.

pub mod media;
pub mod phase;
pub mod snapshot;
pub mod user;

pub use media::Media;
pub use phase::Phase;
pub use snapshot::{Snapshot, SnapshotCounts};
pub use user::Canard;
