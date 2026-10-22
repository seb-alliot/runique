//! `resolve_og_image` — absolute URL on `.with_public_url()`.
use runique::utils::resolve_og_image;

#[test]
fn an_absolute_url_is_kept() {
    for url in ["http://cdn.example/a.png", "https://cdn.example/a.png"] {
        assert_eq!(resolve_og_image(Some("https://www.example.com"), url), url);
        assert_eq!(resolve_og_image(None, url), url);
    }
}

#[test]
fn a_path_is_put_on_the_public_url() {
    assert_eq!(
        resolve_og_image(Some("https://www.example.com"), "/static/og.png"),
        "https://www.example.com/static/og.png"
    );
    // Slashes on either side are joined once.
    assert_eq!(
        resolve_og_image(Some("https://www.example.com/"), "og.png"),
        "https://www.example.com/og.png"
    );
}

#[test]
fn without_a_public_url_the_path_stays_relative() {
    assert_eq!(resolve_og_image(None, "/static/og.png"), "/static/og.png");
    assert_eq!(resolve_og_image(None, "og.png"), "/og.png");
}
