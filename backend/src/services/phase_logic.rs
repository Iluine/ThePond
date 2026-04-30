//! Calcul des paliers à partir du temps serveur.
//!
//! Conformément à PROJECT.md § "Mécanique des paliers" :
//!   - Palier en cours : le dernier dont target_time <= NOW.
//!   - Palier visible : le précédent (par phase_order) du palier en cours.
//!
//! Le calcul est fait en mémoire à partir de la liste complète des
//! paliers (toujours une dizaine au max), ce qui évite des allers-retours
//! SQL et garde la logique trivialement testable.

use crate::models::Phase;

/// Filtre les paliers déjà déclenchés à `now` et retourne celui de
/// `phase_order` maximal — c'est le palier en cours.
pub fn current_phase(phases_all: &[Phase], now: &str) -> Option<Phase> {
    phases_all
        .iter()
        .filter(|p| p.target_time.as_str() <= now)
        .max_by_key(|p| p.phase_order)
        .cloned()
}

/// Le palier visible est le précédent (phase_order strictement inférieur)
/// du palier en cours. None si on est sur le premier palier ou s'il n'y
/// a pas de palier en cours.
pub fn visible_phase(phases_all: &[Phase], current: Option<&Phase>) -> Option<Phase> {
    let current = current?;
    phases_all
        .iter()
        .filter(|p| p.phase_order < current.phase_order)
        .max_by_key(|p| p.phase_order)
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_phase(id: i64, order: i64, time: &str) -> Phase {
        Phase {
            id,
            phase_order: order,
            name: format!("phase-{order}"),
            target_time: time.to_string(),
            triggered_at: None,
            is_final_reveal: false,
        }
    }

    #[test]
    fn current_returns_none_when_no_phases() {
        assert!(current_phase(&[], "2026-06-05T20:00:00Z").is_none());
    }

    #[test]
    fn current_returns_none_when_no_phase_started() {
        let phases = vec![make_phase(1, 0, "2026-06-05T22:00:00Z")];
        assert!(current_phase(&phases, "2026-06-05T20:00:00Z").is_none());
    }

    #[test]
    fn current_picks_latest_started_phase() {
        let phases = vec![
            make_phase(1, 0, "2026-06-05T19:00:00Z"),
            make_phase(2, 1, "2026-06-05T20:30:00Z"),
            make_phase(3, 2, "2026-06-05T22:30:00Z"),
        ];
        let cur = current_phase(&phases, "2026-06-05T21:00:00Z").unwrap();
        assert_eq!(cur.phase_order, 1);
        assert_eq!(cur.name, "phase-1");
    }

    #[test]
    fn visible_is_the_previous_of_current() {
        let phases = vec![
            make_phase(1, 0, "2026-06-05T19:00:00Z"),
            make_phase(2, 1, "2026-06-05T20:30:00Z"),
            make_phase(3, 2, "2026-06-05T22:30:00Z"),
        ];
        let cur = current_phase(&phases, "2026-06-05T23:00:00Z").unwrap();
        let vis = visible_phase(&phases, Some(&cur)).unwrap();
        assert_eq!(cur.phase_order, 2);
        assert_eq!(vis.phase_order, 1);
    }

    #[test]
    fn visible_is_none_on_first_phase() {
        let phases = vec![
            make_phase(1, 0, "2026-06-05T19:00:00Z"),
            make_phase(2, 1, "2026-06-05T20:30:00Z"),
        ];
        let cur = current_phase(&phases, "2026-06-05T19:30:00Z").unwrap();
        assert_eq!(cur.phase_order, 0);
        assert!(visible_phase(&phases, Some(&cur)).is_none());
    }
}
