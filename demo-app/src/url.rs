use crate::views;
use runique::tera::Tera;
use std::sync::Arc;

use runique::{post, urlpatterns, view, Router};

pub fn routes() -> Router<Arc<Tera>> {
    urlpatterns! {
        // index
        "/" => view!{
            GET => views::index
        },
        name ="index",

        // footer links
        "/about" => view!{
            GET => views::about
        },
        name ="about",

        // New form test
        "/test-new-form" => view! {
            GET => views::test_new_form,
            POST => views::test_new_form_submit
        },
        name = "test_new_form",
        // // other links
        "/user" => view! {
            GET => views::form_register_user,
            POST => views::user_profile_submit
        },
        name = "user_profile",


        // search by username
        "/view-user" => view! {
            GET => views::user,
            POST => views::view_user
        },
        name = "view-user",

        // Ajax CSRF test
        "/test-csrf" => post(views::test_csrf),
        name ="test_csrf",


        // // Form with advanced fields
        "/test-fields" => view! {
        GET => views::show_form,
        POST => views::submit_form
        },
        name ="test_fields",
    }
}
