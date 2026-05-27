//! Anti-bot middleware: injects a session-bound random honeypot field name into every request.
//! The field name is generated on first visit and stored in session — no recognizable prefix.
//! On POST, `Request::form()` checks if the field was filled and sets `force_invalid`.
use crate::utils::aliases::AEngine;
use crate::utils::constante::session_key::session::HP_FIELD_KEY;
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};
use tower_sessions::Session;

/// Axum extension injected by the anti-bot middleware.
/// Contains the honeypot field name for this session.
#[derive(Clone, Debug)]
pub struct HoneypotFieldName(pub String);

fn generate_field_name() -> String {
    let bytes: [u8; 8] = rand::random();
    hex::encode(bytes)
}

pub async fn anti_bot_middleware(
    State(_engine): State<AEngine>,
    session: Session,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    let field_name: String = session
        .get::<String>(HP_FIELD_KEY)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(generate_field_name);

    let _ = session.insert(HP_FIELD_KEY, &field_name).await;

    req.extensions_mut().insert(HoneypotFieldName(field_name));
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_field_name_is_hex16() {
        let name = generate_field_name();
        assert_eq!(name.len(), 16, "field name must be 16 hex chars");
        assert!(
            name.chars().all(|c| c.is_ascii_hexdigit()),
            "field name must be hex: {name}"
        );
    }

    #[test]
    fn test_generate_field_name_is_random() {
        let names: std::collections::HashSet<String> =
            (0..20).map(|_| generate_field_name()).collect();
        assert!(names.len() > 1, "field names should be random");
    }

    #[test]
    fn test_honeypot_field_name_clone() {
        let hp = HoneypotFieldName("abc123".to_string());
        let hp2 = hp.clone();
        assert_eq!(hp.0, hp2.0);
    }
}
