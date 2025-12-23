use axum::{
    body::{Body},
    extract::FromRequest,
    http::{Request},
    response::Response,
};
use http_body_util::BodyExt;
use std::collections::HashMap;

pub struct AxumForm<T>(pub T);

impl<S, T> FromRequest<S> for AxumForm<T>
where
    S: Send + Sync,
    T: crate::formulaire::forms_rusti::FormulaireTrait,
{
    type Rejection = Response;


    async fn from_request(mut req: Request<Body>, _state: &S) -> Result<Self, Self::Rejection> {
        // Lire le body brut à chaque fois (ignorer les extensions)
        let bytes = req.body_mut()
            .collect()
            .await
            .map_err(|_| Response::builder().status(400).body(Body::from("Failed to read body")).unwrap())?
            .to_bytes();

        // Parser selon le Content-Type
        let parsed: HashMap<String, String> = match req.headers().get("content-type").and_then(|v| v.to_str().ok()) {
            Some(ct) if ct.starts_with("application/x-www-form-urlencoded") => {
                serde_urlencoded::from_bytes(&bytes).unwrap_or_default()
            }
            Some(ct) if ct.starts_with("application/json") => {
                serde_json::from_slice(&bytes).unwrap_or_default()
            }
            _ => HashMap::new(),
        };

        // Créer et valider le formulaire
        let mut form = T::new();
        form.validate(&parsed);

        Ok(AxumForm(form))
    }
}
