use crate::formulaire::formsrunique::RuniqueForm;
use crate::utils::unmask_csrf_token;
use axum::{body::Body, extract::FromRequest, http::Request, response::Response};
use http_body_util::BodyExt;
use std::collections::HashMap;
use tower_sessions::Session;

pub struct ExtractForm<T>(pub T);

impl<S, T> FromRequest<S> for ExtractForm<T>
where
    S: Send + Sync,
    T: RuniqueForm,
{
    type Rejection = Response;

    async fn from_request(mut req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        // Lire le body brut
        let bytes = req
            .body_mut()
            .collect()
            .await
            .map_err(|_| {
                Response::builder()
                    .status(400)
                    .body(Body::from("Failed to read body"))
                    .unwrap()
            })?
            .to_bytes();

        // Parser le formulaire
        let parsed: HashMap<String, String> = match req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
        {
            Some(ct) if ct.starts_with("application/x-www-form-urlencoded") => {
                serde_urlencoded::from_bytes(&bytes).unwrap_or_default()
            }
            Some(ct) if ct.starts_with("application/json") => {
                serde_json::from_slice(&bytes).unwrap_or_default()
            }
            _ => HashMap::new(),
        };

        // Vérifier CSRF
        let session = match req.extensions().get::<Session>() {
            Some(s) => s,
            None => {
                return Err(Response::builder()
                    .status(500)
                    .body(Body::from("Session middleware missing"))
                    .unwrap())
            }
        };
        let session_token = match session.get::<String>("csrf_token").await {
            Ok(Some(token)) => token,
            _ => {
                return Err(Response::builder()
                    .status(403)
                    .body(Body::from("CSRF token missing in session"))
                    .unwrap())
            }
        };

        // Récupérer le token du formulaire
        let form_token = parsed.get("csrf_token").ok_or_else(|| {
            Response::builder()
                .status(403)
                .body(Body::from("CSRF token missing"))
                .unwrap()
        })?;

        // Décode et démasque le token reçu
        let unmasked_token = match unmask_csrf_token(form_token) {
            Ok(t) => t,
            Err(_) => {
                return Err(Response::builder()
                    .status(403)
                    .body(Body::from("Invalid CSRF token"))
                    .unwrap())
            }
        };

        // Compare avec le token stocké en session
        if unmasked_token != session_token {
            return Err(Response::builder()
                .status(403)
                .body(Body::from("Invalid CSRF token"))
                .unwrap());
        }

        // Comparer avec le token stocké en session
        if unmasked_token != session_token {
            return Err(Response::builder()
                .status(403)
                .body(Body::from("Invalid CSRF token"))
                .unwrap());
        }

        // Créer et valider le formulaire avec RuniqueForm
        let form = T::build_with_data(&parsed);

        Ok(ExtractForm(form))
    }
}
