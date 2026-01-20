use tera::Tera;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;

// On pointe vers le dossier tera_tool de ton arborescence
use crate::request_context::tera_tool::{
    static_balise,
    url_balise,
    balise_csrf_token,
    balise_csp,
    form
};

pub fn register_extensions(
    tera: &mut Tera,
    static_url: String,
    url_registry: Arc<RwLock<HashMap<String, String>>>
) {
    // Enregistrement des filtres statiques
    static_balise::register_all_asset_filters(
        tera,
        static_url.clone(),
        format!("{}/media", static_url),
        static_url.clone(),
        static_url.clone(),
        url_registry.clone(),
    );

    // Enregistrement de la fonction link
    url_balise::register_link_function(tera, url_registry);

    // Enregistrement CSRF
    balise_csrf_token::register_csrf_token(tera);

    // Enregistrement CSP
    tera.register_function("nonce", balise_csp::nonce_function);

    // Enregistrement Formulaire
    tera.register_filter("form_filter", form::form_filter);
}