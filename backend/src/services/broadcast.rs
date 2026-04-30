//! SnapshotHub : broadcast SSE avec debounce.
//!
//! PROJECT.md § "Debounce des updates" :
//!   « Quand plusieurs changements surviennent rapidement (5 invités
//!     postent en 1 seconde), le backend agrège avec un debounce de
//!     300ms et envoie un seul snapshot. »
//!
//! Implémentation : trailing-edge debounce. Les routes mutantes appellent
//! `mark_dirty()`, ce qui pousse un signal sur un mpsc. Une tâche
//! background attend 300 ms de silence après le dernier signal, puis
//! recompute et broadcast.

use std::sync::Arc;
use std::time::Duration;

use sqlx::SqlitePool;
use tokio::sync::{broadcast, mpsc};

use crate::models::Snapshot;
use crate::services::snapshot;

const DEBOUNCE_MS: u64 = 300;
/// Capacité du broadcast channel : si un client lit trop lentement,
/// il "lag" — l'handler SSE traite le RecvError::Lagged en
/// recomputant le snapshot courant pour resynchroniser.
const BROADCAST_CAPACITY: usize = 16;

pub struct SnapshotHub {
    pool: SqlitePool,
    tx: broadcast::Sender<Arc<Snapshot>>,
    dirty_tx: mpsc::UnboundedSender<()>,
}

impl SnapshotHub {
    /// Spawn le hub : retourne un Arc<Self> partageable, et lance la
    /// tâche de debounce en background. La tâche vit aussi longtemps
    /// que le runtime tokio.
    pub fn spawn(pool: SqlitePool) -> Arc<Self> {
        let (tx, _) = broadcast::channel(BROADCAST_CAPACITY);
        let (dirty_tx, dirty_rx) = mpsc::unbounded_channel::<()>();

        let hub = Arc::new(Self {
            pool: pool.clone(),
            tx: tx.clone(),
            dirty_tx,
        });

        tokio::spawn(debounce_loop(pool, tx, dirty_rx));
        hub
    }

    /// Calcule le snapshot courant à la demande (utilisé pour le snapshot
    /// initial envoyé à un nouveau client SSE).
    pub async fn compute_snapshot(&self) -> anyhow::Result<Snapshot> {
        snapshot::compute(&self.pool).await
    }

    /// Souscrit aux broadcasts. Chaque client SSE appelle ceci une fois.
    pub fn subscribe(&self) -> broadcast::Receiver<Arc<Snapshot>> {
        self.tx.subscribe()
    }

    /// Signale qu'un changement a eu lieu — déclenche un recompute
    /// debouncé. Les routes upload / orchestration appelleront ceci
    /// après chaque mutation.
    pub fn mark_dirty(&self) {
        // unbounded_send échoue uniquement si le receiver est drop —
        // dans ce cas le hub ne fonctionne plus et on ignore.
        let _ = self.dirty_tx.send(());
    }
}

/// Boucle de debounce trailing-edge. Attend un premier signal "dirty",
/// puis pour chaque signal supplémentaire reçu pendant la fenêtre,
/// reset le deadline. Quand 300 ms passent sans signal, recompute et
/// broadcast.
async fn debounce_loop(
    pool: SqlitePool,
    tx: broadcast::Sender<Arc<Snapshot>>,
    mut dirty_rx: mpsc::UnboundedReceiver<()>,
) {
    while dirty_rx.recv().await.is_some() {
        let mut deadline = tokio::time::Instant::now() + Duration::from_millis(DEBOUNCE_MS);

        loop {
            tokio::select! {
                _ = tokio::time::sleep_until(deadline) => break,
                msg = dirty_rx.recv() => match msg {
                    Some(_) => {
                        deadline = tokio::time::Instant::now() + Duration::from_millis(DEBOUNCE_MS);
                    }
                    None => return, // channel fermé, arrêt propre
                }
            }
        }

        match snapshot::compute(&pool).await {
            Ok(snap) => {
                // send retourne Err si aucun receiver — pas grave (pas de client connecté)
                let _ = tx.send(Arc::new(snap));
            }
            Err(err) => {
                tracing::error!(%err, "failed to compute snapshot in debounce loop");
            }
        }
    }
}
