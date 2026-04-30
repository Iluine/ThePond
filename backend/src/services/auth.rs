//! Auth HMAC pour les témoins (cf. PROJECT.md § "Authentification").
//!
//! Un seul WITNESS_SECRET par instance, lu depuis l'env. Le token public
//! est dérivé par HMAC-SHA256(secret, "thepond:witness:event:1") puis
//! hex-encodé. Les témoins reçoivent l'URL `/orchestration?token=...`
//! qu'on imprime dans les logs au boot pour que l'admin la partage.
//!
//! Côté backend, l'extractor WitnessAuth lit ?token= dans la query et
//! compare en temps constant. Pas de session, pas de cookie — juste
//! ce token immuable jusqu'à un restart du serveur (qui regénère le
//! secret si non fourni en env).

use std::env;

use axum::extract::{FromRequestParts, Query};
use axum::http::request::Parts;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;

use crate::error::AppError;
use crate::state::AppState;

type HmacSha256 = Hmac<Sha256>;

const TOKEN_PAYLOAD: &str = "thepond:witness:event:1";

/// Récupère ou génère le WITNESS_SECRET. En prod il vient de l'env
/// (compose le pousse depuis .env). En dev sans env, on génère un
/// secret aléatoire UUID-based pour que tout marche out-of-the-box —
/// le token sera juste différent à chaque restart.
pub fn load_or_generate_secret() -> String {
    match env::var("WITNESS_SECRET") {
        Ok(s) if !s.is_empty() => s,
        _ => {
            let gen = uuid::Uuid::new_v4().to_string();
            tracing::warn!(
                "WITNESS_SECRET not set in env — generated a random one for this run; \
                 témoins will need a fresh URL each restart"
            );
            gen
        }
    }
}

/// Calcule le token public à partir du secret. Hex-encoded SHA-256
/// HMAC, 64 chars. Le token est stable tant que le secret ne change pas.
pub fn witness_token(secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(TOKEN_PAYLOAD.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[derive(Deserialize)]
struct TokenQuery {
    token: Option<String>,
}

/// Extractor : tout handler qui prend `WitnessAuth` en argument est
/// protégé. 401 si pas de token ou token invalide.
pub struct WitnessAuth;

#[axum::async_trait]
impl FromRequestParts<AppState> for WitnessAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Query(q) = Query::<TokenQuery>::from_request_parts(parts, state)
            .await
            .map_err(|_| AppError::Unauthorized)?;
        let provided = q.token.unwrap_or_default();
        if provided.is_empty() {
            return Err(AppError::Unauthorized);
        }
        // Comparaison constant-time via le crate `subtle` ré-exporté
        // par hmac : on calcule le HMAC attendu et on vérifie l'égalité
        // sur les bytes hex (qui sont déjà du texte ASCII, donc OK).
        let expected = state.witness_token.as_bytes();
        let provided_bytes = provided.as_bytes();
        if provided_bytes.len() != expected.len() {
            return Err(AppError::Unauthorized);
        }
        let mut diff: u8 = 0;
        for (a, b) in expected.iter().zip(provided_bytes.iter()) {
            diff |= a ^ b;
        }
        if diff != 0 {
            return Err(AppError::Unauthorized);
        }
        Ok(WitnessAuth)
    }
}
