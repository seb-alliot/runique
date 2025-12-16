use rusti::{Router, urlpatterns};
use crate::views;


pub fn urls() -> Router {
    urlpatterns! {
        "/" => get(views::index), name ="index",
        "/about" => get(views::about), name ="about",
    }
}