use axum::{
    response::IntoResponse,
    http::StatusCode,
    extract::Request,
    middleware::Next,
    body::Body,
    response::Response,
};
use tower_sessions::session::Session;
use axum::middleware;
use rusti::middleware::flash_message::FlashMessage;
use rusti::middleware::flash_message::FlashMessageSession;
use rusti::middleware::flash_middleware;


/// Clé de session pour stocker les messages flash
pub const FLASH_MESSAGES_KEY: &str = "flash_messages";

pub async fn test(
    mut req: axum::http::Request<Body>,
    next: Next,
) -> Response {

    // Étape 1 : extraire les messages sans toucher aux extensions ensuite
    let messages = {
        let session = match req.extensions_mut().get_mut::<Session>() {
            Some(s) => s,
            None => return next.run(req).await,
        };

        let messages = session
        .get::<Vec<FlashMessage>>(FLASH_MESSAGES_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_default();

        if !messages.is_empty() {
            // Supprimer les messages après les avoir lus
            let _ = session.remove::<Vec<FlashMessage>>(FLASH_MESSAGES_KEY).await;
        }
        messages
    };
    // Étape 2 : insérer les messages dans les extensions pour le traitement ultérieur
    if !messages.is_empty() {
        req.extensions_mut().insert(messages);
    }
    next.run(req).await
}


/// Handler pour définir un message flash
/// Utilisé pour les tests
async fn set_flash(mut session: Session) -> impl IntoResponse {
    session
        .insert_message(FlashMessage::success("OK"))
        .await
        .unwrap();

    StatusCode::OK
}

async fn read_flash(req: Request) -> impl IntoResponse {
    let messages = req
        .extensions()
        .get::<Vec<FlashMessage>>()
        .cloned()
        .unwrap_or_default();

    (
        StatusCode::OK,
        format!("messages={}", messages.len()),
    )
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        routing::get,
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use tower_sessions::{SessionManagerLayer, MemoryStore};

    #[tokio::test]
    async fn flash_middleware_injects_messages() {
        let store = MemoryStore::default();

        let session_layer = SessionManagerLayer::new(store)
            .with_secure(false)
            .with_expiry(tower_sessions::Expiry::OnSessionEnd);

        let app = Router::new()
            .route("/set", get(set_flash))
            .route("/read", get(read_flash))
            .layer(middleware::from_fn(flash_middleware))
            .layer(session_layer);

        // 1 écrire le flash
        let res1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/set")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res1.status(), StatusCode::OK);

        // récupérer le cookie de session
        let cookie = res1
            .headers()
            .get(axum::http::header::SET_COOKIE)
            .expect("session cookie missing")
            .to_str()
            .unwrap()
            .to_string();

        // 2️ lire le flash AVEC le cookie
        let res2 = app
            .oneshot(
                Request::builder()
                    .uri("/read")
                    .header(axum::http::header::COOKIE, cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let body = axum::body::to_bytes(res2.into_body(), usize::MAX)
            .await
            .unwrap();

        assert_eq!(std::str::from_utf8(&body).unwrap(), "messages=1");
    }
}