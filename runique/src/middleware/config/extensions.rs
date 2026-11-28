//! Puts the engine in the request's extensions, outermost (slot 0): every
//! middleware and extractor inside reads it there, Tera and the config with it.
use crate::{http::RequestExtensions, utils::aliases::AEngine};
use axum::{body::Body, extract::State, http::Request, middleware::Next, response::Response};

pub(crate) async fn extensions_middleware(
    State(engine): State<AEngine>,
    mut req: Request<Body>,
    next: Next,
) -> Response {
    RequestExtensions::new()
        .with_engine(engine)
        .inject_request(&mut req);
    next.run(req).await
}
