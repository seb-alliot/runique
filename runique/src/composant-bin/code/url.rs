use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {

        "/" => view!{ views::index }, name = "index",

        "/inscription" => view! { views::soumission_inscription }, name = "inscription",
        
        "/about" => view! { views::about }, name = "about",
    };
    router
}
