use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

// On pointe vers le dossier tera_tool de ton arborescence
use crate::context::tera::{csp, csrf, form, register_all_asset_filters, register_link_function};

pub fn register_extensions(
    tera: &mut Tera,
    static_url: String,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
) {
    // Enregistrement des filtres statiques
    register_all_asset_filters(
        tera,
        static_url.clone(),
        format!("{}/media", static_url),
        static_url.clone(),
        static_url.clone(),
        url_registry.clone(),
    );

    // Enregistrement de la fonction link
    register_link_function(tera, url_registry);

    // Enregistrement CSRF
    csrf::register_csrf_token(tera);

    // Enregistrement CSP
    tera.register_function("nonce", csp::nonce_function);

    // Enregistrement Formulaire
    tera.register_filter("form_filter", form::form_filter);
}
