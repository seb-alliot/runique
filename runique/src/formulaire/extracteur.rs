use crate::formulaire::formsrunique::RuniqueForm;
use crate::utils::unmask_csrf_token;
use axum::{
    body::Body,
    extract::{FromRef, FromRequest, Multipart},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;
use tower_sessions::Session;

pub struct ExtractForm<T>(pub T);

impl<S, T> FromRequest<S> for ExtractForm<T>
where
    S: Send + Sync,
    T: RuniqueForm,
    Arc<Tera>: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let tera = Arc::<Tera>::from_ref(state);

        // Récupérer le type de contenu
        let content_type = req
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Récupérer la session AVANT de consommer la requête
        let session = req.extensions().get::<Session>().cloned().ok_or_else(|| {
            (StatusCode::INTERNAL_SERVER_ERROR, "Session manquante").into_response()
        })?;

        let mut parsed = HashMap::new();

        // --- EXTRACTION DES DONNÉES ---
        if content_type.starts_with("multipart/form-data") {
            // Mode Multipart
            let mut multipart = Multipart::from_request(req, state).await.map_err(|_| {
                (StatusCode::BAD_REQUEST, "Error parsing multipart").into_response()
            })?;

            while let Ok(Some(field)) = multipart.next_field().await {
                let name = field.name().unwrap_or_default().to_string();
                if let Ok(data) = field.text().await {
                    parsed.insert(name, data);
                }
            }
        } else {
            // Mode URL-encoded ou JSON
            let bytes = req
                .into_body()
                .collect()
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Failed to read body").into_response())?
                .to_bytes();

            if !bytes.is_empty() {
                if content_type.starts_with("application/x-www-form-urlencoded") {
                    parsed = serde_urlencoded::from_bytes(&bytes).unwrap_or_default();
                } else if content_type.starts_with("application/json") {
                    parsed = serde_json::from_slice(&bytes).unwrap_or_default();
                }
            }
        }

        // --- VALIDATION CSRF (pour multipart uniquement) ---
        if content_type.starts_with("multipart/form-data") {
            let session_secret = session
                .get::<String>("csrf_token")
                .await
                .ok()
                .flatten()
                .ok_or_else(|| {
                    (StatusCode::FORBIDDEN, "No CSRF token in session").into_response()
                })?;

            let token_fourni = parsed
                .get("csrf_token")
                .ok_or_else(|| (StatusCode::FORBIDDEN, "CSRF token missing").into_response())?;

            let token_de_la_requete = unmask_csrf_token(token_fourni).map_err(|_| {
                (StatusCode::FORBIDDEN, "Invalid CSRF token format").into_response()
            })?;

            if session_secret != token_de_la_requete {
                return Err((StatusCode::FORBIDDEN, "Invalid CSRF token").into_response());
            }
        }

        // --- CONSTRUCTION DU FORMULAIRE ---
        let mut form = T::build_with_current_data(&parsed, tera.clone());
        form.get_form_mut().set_tera(tera);

        Ok(ExtractForm(form))
    }
}
