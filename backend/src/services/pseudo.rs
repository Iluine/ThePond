//! Génération de pseudo "Canard{Adverbe}{Adjectif}".
//!
//! Format figé par PROJECT.md : "CanardÉlégammentPerché",
//! "CanardJoyeusementDansant", etc. Les listes viennent de pseudo.ron.

use rand::seq::SliceRandom;

use crate::config::PseudoConfig;

pub fn generate(cfg: &PseudoConfig) -> String {
    // Les listes ont été validées non-vides au chargement (config::load).
    let mut rng = rand::thread_rng();
    let adv = cfg.adverbs.choose(&mut rng).expect("adverbs non-empty");
    let adj = cfg.adjectives.choose(&mut rng).expect("adjectives non-empty");
    format!("{}{}{}", cfg.prefix, adv, adj)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> PseudoConfig {
        PseudoConfig {
            prefix: "Canard".into(),
            adverbs: vec!["Élégamment".into(), "Joyeusement".into()],
            adjectives: vec!["Perché".into(), "Dansant".into()],
        }
    }

    #[test]
    fn pseudo_starts_with_prefix() {
        let p = generate(&cfg());
        assert!(p.starts_with("Canard"));
    }

    #[test]
    fn pseudo_contains_one_adverb_and_one_adjective() {
        let p = generate(&cfg());
        let has_adv = ["Élégamment", "Joyeusement"]
            .iter()
            .any(|a| p.contains(a));
        let has_adj = ["Perché", "Dansant"].iter().any(|a| p.contains(a));
        assert!(has_adv, "missing adverb in {p}");
        assert!(has_adj, "missing adjective in {p}");
    }
}
