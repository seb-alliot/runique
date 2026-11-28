//! Absolute image URL resolution (OG image, favicon) on the public URL (`.with_public_url()`).

/// Returns `image` as an absolute URL on `public_url`. An already absolute URL
/// is kept as is; without a public URL the path stays relative.
///
/// Never built from the request's `Host`: that header is the client's to choose.
pub fn resolve_og_image(public_url: Option<&str>, image: &str) -> String {
    if image.starts_with("http://") || image.starts_with("https://") {
        return image.to_string();
    }
    let path = image.trim_start_matches('/');
    match public_url {
        Some(base) => format!("{}/{}", base.trim_end_matches('/'), path),
        None => format!("/{path}"),
    }
}
