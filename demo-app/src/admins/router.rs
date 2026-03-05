use runique::prelude::*;
use runique::{urlpatterns, view};
use crate::admins::handlers;

pub fn admin(prefix: &str) -> Router {
    urlpatterns! {
        &format!("{}/users/list", prefix) => view! { handlers::users_list }, name = "users_list",
        &format!("{}/users/create", prefix) => view! { handlers::users_create }, name = "users_create",
        &format!("{}/users/{{id}}", prefix) => view! { handlers::users_detail }, name = "users_detail",
        &format!("{}/users/{{id}}/edit", prefix) => view! { handlers::users_edit }, name = "users_edit",
        &format!("{}/users/{{id}}/delete", prefix) => view! { handlers::users_delete }, name = "users_delete",
    }
}
