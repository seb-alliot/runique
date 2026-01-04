//! HTTP response helpers
//!
//! This module provides convenient functions for creating common HTTP responses
//! including HTML pages, JSON responses, redirects, and error pages.
//!
//! # Examples
//!
//! ```rust
//! use runique::response::{html_response, json_response, redirect};
//! use runique::StatusCode;
//! use serde_json::json;
//!
//! // HTML response
//! let html = html_response(StatusCode::OK, "<h1>Hello</h1>");
//!
//! // JSON response
//! let json = json_response(StatusCode::OK, json!({"status": "ok"}));
//!
//! // Redirect
//! let redir = redirect("/home");
//! ```

use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tera::Tera;

/// Renders a 404 Not Found page
///
/// Attempts to render a custom "404" template if available,
/// otherwise falls back to a default styled 404 page.
///
/// # Arguments
///
/// * `tera` - Template engine instance
///
/// # Examples
///
/// ```rust,no_run
/// # use std::sync::Arc;
/// # use axum::extract::Extension;
/// use runique::response::render_404;
/// use tera::Tera;
///
/// async fn fallback_handler(Extension(tera): Extension<Arc<Tera>>) -> axum::response::Response {
///     render_404(&tera)
/// }
/// ```
pub fn render_404(tera: &Tera) -> Response {
    let context = tera::Context::new();
    // Try to render custom 404 template
    if let Ok(html) = tera.render("404", &context) {
        return (StatusCode::NOT_FOUND, Html(html)).into_response();
    }
    // Fallback to hardcoded 404
    fallback_404_html()
}

/// Returns a fallback HTML page for 404 errors
///
/// This is used when no custom 404 template is available.
/// Provides a clean, styled error page with a "Go to Homepage" button.
///
/// # Examples
///
/// ```rust
/// use runique::response::fallback_404_html;
///
/// let response = fallback_404_html();
/// ```
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

/// Creates a JSON response
///
/// Serializes the provided data to JSON and returns it with the given status code.
///
/// # Arguments
///
/// * `status` - HTTP status code
/// * `data` - JSON data to serialize
///
/// # Examples
///
/// ```rust
/// use runique::response::json_response;
/// use serde_json::json;
/// use runique::StatusCode;
///
/// let response = json_response(
///     StatusCode::OK,
///     json!({ "message": "Success", "count": 42 })
/// );
/// ```
pub fn json_response(status: StatusCode, data: serde_json::Value) -> Response {
    use axum::Json;
    (status, Json(data)).into_response()
}

/// Creates a simple HTML response
///
/// Wraps the provided HTML content and returns it with the given status code.
///
/// # Arguments
///
/// * `status` - HTTP status code
/// * `html` - HTML content
///
/// # Examples
///
/// ```rust
/// use runique::response::html_response;
/// use runique::StatusCode;
///
/// let response = html_response(
///     StatusCode::OK,
///     "<h1>Hello, World!</h1>"
/// );
/// ```
pub fn html_response(status: StatusCode, html: impl Into<String>) -> Response {
    (status, Html(html.into())).into_response()
}

/// Creates a redirect response
///
/// Returns a 303 See Other redirect to the specified URI.
///
/// # Arguments
///
/// * `uri` - Destination URI for the redirect
///
/// # Examples
///
/// ```rust
/// use runique::response::redirect;
///
/// // Redirect to login page
/// let response = redirect("/login");
///
/// // Redirect to external URL
/// let response = redirect("https://example.com");
/// ```
pub fn redirect(uri: &str) -> Response {
    use axum::response::Redirect;
    Redirect::to(uri).into_response()
}
