use crate::views;
use runique::prelude::*;
use runique::{urlpatterns, view}; // Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ GET => views::index }, name = "index",

        "/about" => view! { GET => views::about }, name = "about",
        "/inscription" => view! { GET => views::inscription, POST => views::soumission_inscription }, name = "inscription",
        "/view-user" => view! { GET => views::search_user_form }, name = "view-user-form",
        "/view-user" => view! { POST => views::info_user }, name = "search-user",
        "/blog" => view! { GET => views::blog_form, POST => views::blog_save }, name = "blog_info",
        "/test-csrf" => view! { POST => views::test_csrf }, name = "test_csrf",
        "/upload-image" => view! { GET => views::upload_image_form, POST => views::upload_image_submit }, name = "upload_image",
    };
    router
}
