use rusti::{Router, urlpatterns};
use crate::views;

pub fn urls() -> Router {
    urlpatterns! {
        // Get requests
        "/" => get(views::index), name ="index",
        "/about" => get(views::about), name ="about",
        "/user/{id}/{name}" => get(views::user_profile) ,name ="user_profile",
        "/sapin" => get(views::about_sapin), name ="about_sapin",
        // Post requests
        "/user/{id}/{name}" => post(views::user_profile_submit), name ="user_profile_submit",
        "/test-csrf" => post(views::test_csrf), name ="test_csrf",
    }
}