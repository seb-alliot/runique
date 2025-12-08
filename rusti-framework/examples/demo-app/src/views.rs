use rusti::{
    Extension,
    Response,
    StatusCode,
    Context,
    Tera,
    Settings,
};


use rusti::middleware::error_handler::render_template;
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

    render_template(&tera, "base.html", &context, StatusCode::OK, &config)
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

    render_template(&tera, "about/about.html", &context, StatusCode::OK, &config)
}
