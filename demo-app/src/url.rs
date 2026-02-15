use runique::prelude::*;
use runique::{urlpatterns, view}; // <= Macros explicites

pub fn routes() -> Router {
    let router = urlpatterns! {
        "/" => view!{ views::index }, name = "index",

        "/about" => view! { views::about }, name = "about",
        "/inscription" => view! { views::soumission_inscription }, name = "inscription",
        "/view-user" => view! { views::info_user }, name = "search-user",
        "/blog" => view! { views::blog_save }, name = "blog_info",
        "/test-csrf" => view! { views::test_csrf }, name = "test_csrf",
        "/upload-image" => view! { views::upload_image_submit }, name = "upload_image",
        "/test-fields" => view! { views::test_fields }, name = "test_fields",
    };
    router
}
