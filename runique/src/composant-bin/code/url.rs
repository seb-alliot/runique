use crate::views::*;
use runique::prelude::*;

/// Example route table wiring `/`, `/inscription`, `/connexion` and `/about` to their views.
pub fn routes() -> Router {
    urlpatterns! {
        "/"            => view!{ index },                  name = "index",
        "/inscription" => view!{ soumission_inscription }, name = "inscription",
        "/connexion"   => view!{ connexion },              name = "connexion",
        "/about"       => view!{ about },                  name = "about",
    }
}
