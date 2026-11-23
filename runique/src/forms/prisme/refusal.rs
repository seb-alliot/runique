//! Why a request body was refused, kept small until it becomes the response.
use crate::errors::error::ErrorContext;
use crate::middleware::errors::error::too_large;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// A body Aegis refused. The response is only built when Axum asks for it:
/// a `Response` as the error of every function reading the body would make
/// each of their results as large as a response, refused or not.
#[derive(Debug)]
pub enum BodyRefusal {
    /// A status and its message as the body.
    Status(StatusCode, String),
    /// A 413 of the framework: the message is also the text of the error page.
    TooLarge(String),
    /// An error with its context, for the error page or the debug page.
    Context(Box<ErrorContext>),
}

impl IntoResponse for BodyRefusal {
    fn into_response(self) -> Response {
        match self {
            Self::Status(status, message) => (status, message).into_response(),
            Self::TooLarge(message) => too_large(message),
            Self::Context(context) => (*context).into_error_response(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::middleware::errors::error::FrameworkTooLarge;
    use http_body_util::BodyExt;
    use std::sync::Arc;

    async fn body(response: Response) -> String {
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        String::from_utf8(bytes.to_vec()).unwrap()
    }

    // The response a refusal becomes is the one the body readers used to
    // build on the spot: same status, same body, nothing attached.
    #[tokio::test]
    async fn a_status_refusal_is_its_status_and_its_message() {
        for status in [
            StatusCode::FORBIDDEN,
            StatusCode::BAD_REQUEST,
            StatusCode::INTERNAL_SERVER_ERROR,
        ] {
            let response = BodyRefusal::Status(status, "why".to_string()).into_response();
            let expected = (status, "why".to_string()).into_response();

            assert_eq!(response.status(), status);
            assert_eq!(response.headers(), expected.headers());
            assert!(response.extensions().get::<FrameworkTooLarge>().is_none());
            assert!(response.extensions().get::<Arc<ErrorContext>>().is_none());
            assert_eq!(body(response).await, "why");
        }
    }

    // The 413 keeps the mark the error middleware reads to render its page.
    #[tokio::test]
    async fn a_too_large_refusal_is_the_framework_413() {
        let response = BodyRefusal::TooLarge("too big".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
        assert!(response.extensions().get::<FrameworkTooLarge>().is_some());
        assert!(response.extensions().get::<Arc<ErrorContext>>().is_none());
        assert_eq!(body(response).await, "too big");
    }

    // The context travels with a bare response of its status: the error
    // middleware renders the page from it.
    #[tokio::test]
    async fn a_context_refusal_carries_its_context() {
        let context = ErrorContext::generic(StatusCode::INTERNAL_SERVER_ERROR, "no folder");
        let response = BodyRefusal::Context(Box::new(context)).into_response();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let carried = response
            .extensions()
            .get::<Arc<ErrorContext>>()
            .expect("the context");
        assert_eq!(carried.message, "no folder");
        assert_eq!(carried.status_code, 500);
        assert!(response.extensions().get::<FrameworkTooLarge>().is_none());
        assert_eq!(body(response).await, "");
    }

    // What the whole change is for: a refusal stays far under the size of a
    // response.
    #[test]
    fn a_refusal_is_small() {
        assert!(std::mem::size_of::<BodyRefusal>() <= 40);
        assert!(std::mem::size_of::<BodyRefusal>() < std::mem::size_of::<Response>());
    }
}
