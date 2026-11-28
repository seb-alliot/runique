//! The 413 Runique answers when a body goes over one of its own limits.
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Marks a 413 Runique built itself (a body over one of its limits), with
/// its translated reason: only those get `413.html`, which shows the reason;
/// an API's own 413 keeps its body.
#[derive(Clone)]
pub(crate) struct FrameworkTooLarge(pub(crate) String);

/// A 413 built by Runique: `message` as the body for an HTTP client, and as
/// the text of the page `error_handler_middleware` renders for a browser.
pub(crate) fn too_large(message: String) -> Response {
    let mut res = (StatusCode::PAYLOAD_TOO_LARGE, message.clone()).into_response();
    res.extensions_mut().insert(FrameworkTooLarge(message));
    res
}
