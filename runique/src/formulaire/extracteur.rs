use crate::formulaire::formsrunique::RuniqueForm;
use crate::utils::unmask_csrf_token;
use axum::{body::Body, extract::FromRef, extract::FromRequest, http::Request, response::Response};
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

    async fn from_request(mut req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let tera = Arc::<Tera>::from_ref(state);

        // 2. Lire le body brut
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

        if bytes.is_empty() {
            let mut form = T::build(tera.clone());
            form.get_form_mut().set_tera(tera);
            return Ok(ExtractForm(form));
        }

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

        let session = req.extensions().get::<Session>().ok_or_else(|| {
            Response::builder()
                .status(500)
                .body(Body::from("Session middleware missing"))
                .unwrap()
        })?;

        let session_token = session
            .get::<String>("csrf_token")
            .await
            .map_err(|_| {
                Response::builder()
                    .status(500)
                    .body(Body::from("Session error"))
                    .unwrap()
            })?
            .ok_or_else(|| {
                Response::builder()
                    .status(403)
                    .body(Body::from("CSRF token missing in session"))
                    .unwrap()
            })?;

        // 6. Validation du token
        let form_token = parsed.get("csrf_token").ok_or_else(|| {
            Response::builder()
                .status(403)
                .body(Body::from("CSRF token missing"))
                .unwrap()
        })?;

        let unmasked_token = unmask_csrf_token(form_token).map_err(|_| {
            Response::builder()
                .status(403)
                .body(Body::from("Invalid CSRF token masking"))
                .unwrap()
        })?;

        if unmasked_token != session_token {
            return Err(Response::builder()
                .status(403)
                .body(Body::from("Invalid CSRF token mismatch"))
                .unwrap());
        }

        let mut form = T::build_with_current_data(&parsed, tera.clone());
        form.get_form_mut().set_tera(tera);

        Ok(ExtractForm(form))
    }
}
