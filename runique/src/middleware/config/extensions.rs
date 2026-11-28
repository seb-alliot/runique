//! Puts the engine in the request's extensions, outermost (slot 0): every
//! middleware and extractor inside reads it there, Tera and the config with it.
use crate::{http::RequestExtensions, utils::aliases::AEngine};
use axum::extract::State;

pub(crate) async fn extensions_middleware(
    State(engine): State<AEngine>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    RequestExtensions::new()
        .with_engine(engine)
        .inject_request(&mut req);
    next.run(req).await
}
