use rusti::{Router, urlpatterns, view , post};
use crate::views;

pub fn urls() -> Router {
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

        // sapin page
        "/sapin" => view!{
            GET => views::about_sapin
        },
        name ="about_sapin",

        // other links
        "/user" => view! {
            GET => views::user_profile,
            POST => views::user_profile_submit
        }, name = "user_profile",

        "/view-user" => view! {
            GET => views::user,
            POST => views::view_user
        }, name = "view-user",

        // Ajax CSRF test
        "/test-csrf" => post(views::test_csrf), name ="test_csrf",
    }
}