use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::Tera;

// On pointe vers le dossier tera_tool de ton arborescence
pub use crate::context::tera::register_all_asset_filters;

pub fn register_extensions(
    tera: &mut Tera,
    static_url: String,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
) {
    // Enregistrement centralisé de tous les filtres et fonctions
    register_all_asset_filters(
        tera,
        static_url.clone(),
        format!("{}/media", static_url),
        static_url.clone(),
        static_url.clone(),
        url_registry,
    );
}
