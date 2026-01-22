use std::collections::HashMap;
use tera::{Function, Result as TeraResult, Value};

use async_trait::async_trait;
use axum::{
    body::Body,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tower_sessions::session::Error as SessionError;
use tower_sessions::Session;

use crate::utils::csrf::CsrfToken;

/// Fonction Tera pour générer le champ CSRF dans les formulaires HTML
/// Cette fonction peut être utilisée dans tes templates Tera comme `{{ csrf_token() }}`
/// Elle injecte un `<input type="hidden" name="csrf_token">` avec le token passé ou par défaut.
pub struct CsrfTokenFunction;

// Clé utilisée pour stocker le token CSRF dans la session
const CSRF_TOKEN_KEY: &str = "csrf_token";

impl Function for CsrfTokenFunction {
    /// Indique que la fonction peut être utilisée en toute sécurité dans du HTML
    fn is_safe(&self) -> bool {
        true
    }

    /// Génère le champ HTML pour le token CSRF
    /// `args` peut contenir un token spécifique à injecter (clé "token") ou "csrf_token" par défaut
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let token_str = args.get("token")
            .and_then(|v| v.as_str())
            .or_else(|| {
                // fallback sur "csrf_token" si "token" n'est pas passé
                args.get("csrf_token")
                    .and_then(|v| v.as_str())
            })
            .unwrap_or(""); // si aucun token trouvé, mettre une chaîne vide pour éviter panic

        Ok(Value::String(format!(
            r#"<input type="hidden" name="csrf_token" value="{}">"#,
            token_str
        )))
    }
}

/// Enregistre la fonction CSRF dans Tera pour pouvoir l'utiliser dans les templates
pub fn register_csrf_token(tera: &mut tera::Tera) {
    tera.register_function("csrf_token", CsrfTokenFunction);
}

#[async_trait]
/// Trait pour simplifier l'insertion d'un CsrfToken dans une session Tower
pub trait SendCsrfToken {
    async fn insert_csrf_token(&self, token: CsrfToken) -> Result<(), SessionError>;
}

#[async_trait]
impl SendCsrfToken for Session {
    /// Insère le token CSRF dans la session sous la clé CSRF_TOKEN_KEY
    async fn insert_csrf_token(&self, token: CsrfToken) -> Result<(), SessionError> {
        self.insert(CSRF_TOKEN_KEY, token).await
    }
}

/// Erreur custom pour le middleware/extracteur CSRF
/// Permet à Axum de retourner une réponse HTTP si la récupération du token échoue
pub struct CsrfContextError(SessionError);

impl IntoResponse for CsrfContextError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erreur lors de la lecture du token CSRF: {}", self.0),
        )
            .into_response()
    }
}

/// Extracteur/middleware pour récupérer le token CSRF depuis la session
/// - Vérifie que la session existe dans les extensions
/// - Récupère le token CSRF brut depuis la session
/// - Masque le token pour l'injection dans templates ou headers
/// - Insère le token masqué dans `req.extensions` pour usage ultérieur
pub async fn extract_csrf_token(mut req: axum::http::Request<Body>, next: Next) -> Response {
    let token = {
        // On récupère la session depuis les extensions Axum
        let session = match req.extensions().get::<Session>() {
            Some(s) => s,
            None => return next.run(req).await, // Pas de session : passer au handler suivant
        };

        // On tente de récupérer le token CSRF brut depuis la session
        match session.get::<CsrfToken>(CSRF_TOKEN_KEY).await {
            Ok(Some(t)) => Some(t),
            _ => None,
        }
    };

    if let Some(t) = token {
        // Masquer le token pour éviter de divulguer le token brut côté client (prévention BREACH)
        let masked_token = t.masked();
        // Stocker le token masqué dans les extensions pour qu'il soit accessible dans templates
        req.extensions_mut().insert(masked_token);
    }

    // Passer la requête au handler suivant
    next.run(req).await
}
