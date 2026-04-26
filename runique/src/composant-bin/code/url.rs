use crate::views::*;
use runique::prelude::*;

pub fn routes() -> Router {
    urlpatterns! {
        "/"            => view!{ index },                  name = "index",
        "/inscription" => view!{ soumission_inscription }, name = "inscription",
        "/about"       => view!{ about },                  name = "about",
    }
}
