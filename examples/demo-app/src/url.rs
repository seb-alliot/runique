use rusti::{Router, urlpatterns, get, post};
use crate::views;

pub fn urls() -> Router {
    urlpatterns! {
        // index
        "/" => get(views::index), name ="index",

        // footer links
        "/about" => get(views::about), name ="about",
        "/sapin" => get(views::about_sapin), name ="about_sapin",

        // other links
        "/user/{id}/{name}" => get(views::user_profile).post(views::user_profile_submit), name = "user_profile",

        // Ajax CSRF test
        "/test-csrf" => post(views::test_csrf), name ="test_csrf",
    }
}