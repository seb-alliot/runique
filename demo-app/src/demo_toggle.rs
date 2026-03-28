use runique::admin::router::admin_router::ADMIN_TEMPLATE_SESSION_KEY;
use runique::axum::{
    Router,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use runique::tower_sessions::Session;

/// Route de bascule de template dashboard — spécifique à la démo.
///
/// - Sans override session → passe au template Runique par défaut
/// - Avec override session → supprime l'override (retour au template configuré)
async fn toggle_dashboard_template(session: Session) -> Response {
    let current: Option<String> = session
        .get::<String>(ADMIN_TEMPLATE_SESSION_KEY)
        .await
        .unwrap_or(None);

    if current.is_some() {
        let _ = session.remove::<String>(ADMIN_TEMPLATE_SESSION_KEY).await;
    } else {
        let _ = session
            .insert(ADMIN_TEMPLATE_SESSION_KEY, "admin/dashboard")
            .await;
    }

    Redirect::to("/admin/").into_response()
}

pub fn router(prefix: &str) -> Router {
    let p = prefix.trim_end_matches('/');
    Router::new().route(
        &format!("{}/toggle-template", p),
        get(toggle_dashboard_template),
    )
}
