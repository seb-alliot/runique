use rusti::{
    Extension,
    Response,
    StatusCode,
    Context,
    Tera,
    Settings,
};


use rusti::middleware_folder::error_handler::return_redirect;
use std::sync::Arc;
use serde_json::json;

/// Page d'accueil
pub async fn index(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "Bienvenue sur Rusti",
        "description": "Un framework web moderne inspiré de Django",
        "debug": config.debug,
    })).unwrap_or_default();

    return_redirect(&tera, "base1.html", &context, StatusCode::OK, &config)
}

/// Page "À propos"
pub async fn about(
    Extension(tera): Extension<Arc<Tera>>,
    Extension(config): Extension<Arc<Settings>>,
) -> Response {
    let context = Context::from_serialize(json!({
        "title": "À propos de Rusti",
        "debug": config.debug,
    })).unwrap_or_default();

    return_redirect(&tera, "about/about.html", &context, StatusCode::OK, &config)
}
