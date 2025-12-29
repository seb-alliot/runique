// rusti/src/processor/nonce_processor.rs

use tera::Context;
use axum::extract::Request;

/// Injecte le nonce CSP dans le contexte Tera
///
/// Appelé automatiquement par le Template extractor
pub fn inject_nonce_into_context(context: &mut Context, request: &Request) {
    // Récupérer le nonce depuis les extensions de la requête
    if let Some(nonce) = request.extensions().get::<String>() {
        context.insert("csp_nonce", nonce);
    }
}