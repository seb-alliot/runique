use rusti::{Router, urlpatterns};
use crate::views;


pub fn urls() -> Router {
    urlpatterns! {
        "/" => get(views::index), name ="index",
        "/about" => get(views::about), name ="about",
        "/user/{id}/{name}" => get(views::user_profile), name ="user_profile",
    }
}