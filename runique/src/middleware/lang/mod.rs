//! The language of each visitor: read from the `Accept-Language` header of
//! the request, for the time of that request.
use crate::translation::{Lang, with_lang};
use axum::{
    body::Body,
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

/// Answers the request in the language its browser asks for, when Runique or
/// the project has it; in the application's own otherwise.
///
/// The header only ever picks a loaded language: its text is never written
/// back. Every answer says `Vary: Accept-Language`, so that a cache between
/// the visitor and the application never serves a page to someone who asked
/// for another language.
pub async fn lang_middleware(req: Request<Body>, next: Next) -> Response {
    let asked = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|value| value.to_str().ok())
        .and_then(Lang::from_accept_language);

    let mut response = match asked {
        Some(lang) => with_lang(lang, next.run(req)).await,
        None => next.run(req).await,
    };
    // Added, never set: compression has its own `Vary: Accept-Encoding`.
    response
        .headers_mut()
        .append(header::VARY, HeaderValue::from_static("accept-language"));
    response
}
