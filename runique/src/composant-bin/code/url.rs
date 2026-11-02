use crate::views::*;
use runique::prelude::*;

/// The application's routes: each one named, so templates link to it with
/// `{% link "name" %}` instead of writing the path.
pub fn routes() -> Routes {
    urlpatterns! {
        "/"        => view!{ index },   name = "index",
        "/signup"  => view!{ sign_up }, name = "signup",
        "/login"   => view!{ sign_in }, name = "login",
        // POST only: signing out changes the session, a GET never should.
        "/logout"  => post(sign_out),   name = "logout",
        "/about"   => view!{ about },   name = "about",
    }
}
