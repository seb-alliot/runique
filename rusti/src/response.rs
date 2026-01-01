use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Tera;

/// Module contenant les helpers pour créer des réponses HTTP
/// Rend une page 404 simple
///
/// # Exemple
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use axum::extract::Extension;
/// use rusti::response::render_404;
/// use tera::Tera;
///
/// async fn fallback_handler(Extension(tera): Extension<Arc<Tera>>) -> axum::response::Response {
///     render_404(&tera)
/// }
/// ```
pub fn render_404(tera: &Tera) -> Response {
    let context = tera::Context::new();

    // Essayer de rendre le template 404 personnalisé
    if let Ok(html) = tera.render("404", &context) {
        return (StatusCode::NOT_FOUND, Html(html)).into_response();
    }

    // Fallback 404 en dur
    fallback_404_html()
}

/// Fallback HTML pour erreur 404
pub fn fallback_404_html() -> Response {
    let html = r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>404 - Page Not Found</title>
            <style>
                * { margin: 0; padding: 0; box-sizing: border-box; }
                body {
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    min-height: 100vh;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    padding: 20px;
                }
                .container {
                    background: white;
                    border-radius: 16px;
                    padding: 60px 40px;
                    max-width: 600px;
                    text-align: center;
                    box-shadow: 0 20px 60px rgba(0,0,0,0.3);
                }
                .icon { font-size: 80px; margin-bottom: 20px; }
                h1 { color: #333; font-size: 32px; margin-bottom: 16px; }
                p { color: #666; font-size: 18px; margin-bottom: 30px; }
                .btn {
                    display: inline-block;
                    padding: 12px 30px;
                    background: #667eea;
                    color: white;
                    text-decoration: none;
                    border-radius: 8px;
                    font-weight: 600;
                    transition: transform 0.2s;
                }
                .btn:hover { transform: translateY(-2px); }
            </style>
        </head>
        <body>
            <div class="container">
                <div class="icon">🔍</div>
                <h1>Page Not Found</h1>
                <p>The page you're looking for doesn't exist.</p>
                <a href="/" class="btn">Go to Homepage</a>
            </div>
        </body>
        </html>"#;

    (StatusCode::NOT_FOUND, Html(html)).into_response()
}

/// Crée une réponse JSON
///
/// # Exemple
/// ```rust
/// use rusti::response::json_response;
/// use serde_json::json;
/// use rusti::StatusCode;
///
/// let response = json_response(
///     StatusCode::OK,
///     json!({ "message": "Success" })
/// );
/// ```
pub fn json_response(status: StatusCode, data: serde_json::Value) -> Response {
    use axum::Json;
    (status, Json(data)).into_response()
}

/// Crée une réponse HTML simple
///
/// # Exemple
/// ```rust
/// use rusti::response::html_response;
/// use rusti::StatusCode;
///
/// let response = html_response(
///     StatusCode::OK,
///     "<h1>Hello, World!</h1>"
/// );
/// ```
pub fn html_response(status: StatusCode, html: impl Into<String>) -> Response {
    (status, Html(html.into())).into_response()
}

/// Crée une réponse de redirection
///
/// # Exemple
/// ```rust
/// use rusti::response::redirect;
///
/// let response = redirect("/login");
/// ```
pub fn redirect(uri: &str) -> Response {
    use axum::response::Redirect;
    Redirect::to(uri).into_response()
}
