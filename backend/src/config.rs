use anyhow::{Context, Result};
use serde::Deserialize;
use std::{env, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
pub struct Theme {
    pub event_name: String,
    pub expected_guests_count: u32,
    pub witness_secret_env_var: String,
}

/// Configuration du générateur de pseudo, chargée depuis pseudo.ron
/// du dossier du thème.
#[derive(Debug, Deserialize, Clone)]
pub struct PseudoConfig {
    pub prefix: String,
    pub adverbs: Vec<String>,
    pub adjectives: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_address: String,
    pub theme_path: PathBuf,
    pub db_path: PathBuf,
    pub uploads_path: PathBuf,
    pub theme: Theme,
    pub pseudo: PseudoConfig,
}

impl Config {
    pub fn load() -> Result<Self> {
        let bind_address =
            env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        let theme_path = PathBuf::from(
            env::var("THEME_PATH")
                .unwrap_or_else(|_| "../themes/duck-pond/theme.ron".to_string()),
        );
        let db_path = PathBuf::from(env::var("DB_PATH").unwrap_or_else(|_| "thepond.db".to_string()));
        let uploads_path =
            PathBuf::from(env::var("UPLOADS_PATH").unwrap_or_else(|_| "uploads".to_string()));

        let theme_text = std::fs::read_to_string(&theme_path)
            .with_context(|| format!("failed to read theme file at {}", theme_path.display()))?;
        let theme: Theme = ron::from_str(&theme_text)
            .with_context(|| format!("failed to parse RON theme at {}", theme_path.display()))?;

        // pseudo.ron est dans le même dossier que theme.ron
        let theme_dir = theme_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("theme path has no parent"))?;
        let pseudo_path = theme_dir.join("pseudo.ron");
        let pseudo_text = std::fs::read_to_string(&pseudo_path).with_context(|| {
            format!("failed to read pseudo file at {}", pseudo_path.display())
        })?;
        let pseudo: PseudoConfig = ron::from_str(&pseudo_text).with_context(|| {
            format!("failed to parse RON pseudo at {}", pseudo_path.display())
        })?;
        if pseudo.adverbs.is_empty() || pseudo.adjectives.is_empty() {
            anyhow::bail!("pseudo lists must be non-empty (got {} adverbs, {} adjectives)",
                pseudo.adverbs.len(), pseudo.adjectives.len());
        }

        Ok(Self {
            bind_address,
            theme_path,
            db_path,
            uploads_path,
            theme,
            pseudo,
        })
    }
}
