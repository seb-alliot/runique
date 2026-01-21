/// Response Helper Functions
///
/// Fonctions utilitaires pour créer des réponses HTTP.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

/// Crée une réponse JSON avec un statut personnalisé
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::json_response;
/// use axum::http::StatusCode;
/// use serde_json::json;
///
/// let response = json_response(
///     StatusCode::CREATED,
///     json!({ "message": "User created", "id": 1 })
/// );
/// ```
pub fn json_response(status: StatusCode, data: Value) -> Response {
    (status, Json(data)).into_response()
}

/// Crée une réponse JSON d'erreur
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::json_error;
/// use axum::http::StatusCode;
///
/// let response = json_error(StatusCode::NOT_FOUND, "User not found");
/// ```
pub fn json_error(status: StatusCode, message: &str) -> Response {
    json_response(status, json!({ "error": message }))
}

/// Crée une réponse JSON de succès
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::json_success;
/// use serde_json::json;
///
/// let response = json_success("User created successfully", json!({ "id": 1 }));
/// ```
pub fn json_success(message: &str, data: Value) -> Response {
    json_response(
        StatusCode::OK,
        json!({
            "status": "success",
            "message": message,
            "data": data
        }),
    )
}

/// Crée une réponse HTML avec un statut personnalisé
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::html_response;
/// use axum::http::StatusCode;
///
/// let response = html_response(StatusCode::OK, "<h1>Hello</h1>");
/// ```
pub fn html_response(status: StatusCode, html: &str) -> Response {
    (
        status,
        [("Content-Type", "text/html; charset=utf-8")],
        html.to_string(),
    )
        .into_response()
}

/// Crée une réponse texte brut
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::text_response;
/// use axum::http::StatusCode;
///
/// let response = text_response(StatusCode::OK, "Plain text content");
/// ```
pub fn text_response(status: StatusCode, text: &str) -> Response {
    (
        status,
        [("Content-Type", "text/plain; charset=utf-8")],
        text.to_string(),
    )
        .into_response()
}

/// Crée une réponse de redirection
///
/// # Exemple
/// ```rust,no_run
/// use runique::utils::response_helpers::redirect;
///
/// let response = redirect("/home");
/// ```
pub fn redirect(uri: &str) -> Response {
    (StatusCode::FOUND, [("Location", uri)]).into_response()
}

/// Crée une réponse de redirection permanente (301)
pub fn redirect_permanent(uri: &str) -> Response {
    (StatusCode::MOVED_PERMANENTLY, [("Location", uri)]).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_response() {
        let resp = json_response(StatusCode::OK, json!({ "test": "data" }));
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn test_json_error() {
        let resp = json_error(StatusCode::BAD_REQUEST, "Invalid input");
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_redirect() {
        let resp = redirect("/home");
        assert_eq!(resp.status(), StatusCode::FOUND);
    }
}
