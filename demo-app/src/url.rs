use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",
        "/about" => view! { GET => views::about }, name = "about",
        // "/user" => view! { GET => views::inscription, POST => views::soumission_inscription }, name = "user_profile",
        "/view-user" => view! { GET => views::search_user_form }, name = "view-user-form",
        "/view-user" => view! { POST => views::info_user }, name = "view-user",
        "/blog" => view! { GET => views::blog_form }, name = "blog_info",
        "/test-csrf" => view! { POST => views::test_csrf }, name = "test_csrf",
    };
    router
}
