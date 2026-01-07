use crate::views;
use runique::{post, urlpatterns, view, Router};

pub fn routes() -> Router {
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

        // other links
        "/user" => view! {
            GET => views::form_register_user,
            POST => views::user_profile_submit
        }, name = "user_profile",


        // search by username
        "/view-user" => view! {
            GET => views::user,
            POST => views::view_user
        }, name = "view-user",

        // Ajax CSRF test
        "/test-csrf" => post(views::test_csrf), name ="test_csrf",
    }
}
