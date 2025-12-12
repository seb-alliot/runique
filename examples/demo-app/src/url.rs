use rusti::{Router};
use rusti::axum::routing::get;

use crate::views;

pub fn urls() -> Router {
    Router::new()
        .route("/", get(views::index))
        .route("/about", get(views::about))
}