use crate::forms::field::RuniqueForm;
use crate::forms::utils::extractor::Prisme;
use axum::response::Response;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

/// CSRF gate : vérification du token dans les données parsées.
/// Retourne Some(Prisme) si invalid/missing (formulaire vide avec erreur), None sinon.
pub async fn csrf_gate<T: RuniqueForm>(
    parsed: &HashMap<String, Vec<String>>,
    csrf_session: &str,
    tera: Arc<Tera>,
) -> Result<Option<Prisme<T>>, Response> {
    let csrf_submitted = parsed
        .get("csrf_token")
        .and_then(|v| v.last())
        .map(|s| s.as_str());

    if csrf_submitted != Some(csrf_session) {
        let empty: HashMap<String, String> = HashMap::new();
        let mut form = T::build_with_data(&empty, tera.clone(), csrf_session).await;
        form.get_form_mut().set_tera(tera);
        if let Some(csrf_field) = form.get_form_mut().fields.get_mut("csrf_token") {
            csrf_field.set_error("Token CSRF invalide ou manquant".to_string());
        }
        return Ok(Some(Prisme(form)));
    }

    Ok(None)
}
