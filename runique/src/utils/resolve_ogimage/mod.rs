//! OG image absolute URL resolution — host extracted from `HostPolicy` according to debug/prod mode.
use crate::middleware::allowed_hosts::HostPolicy;

/// Returns the absolute OG image URL based on the context (debug/prod).
pub fn resolve_og_image(security: &HostPolicy, debug: bool, og_image: &str) -> String {
    let host = if debug {
        security
            .allowed_hosts
            .iter()
            .find(|h| h.contains("localhost") || h.contains("127.0.0.1"))
            .map(|h| format!("http://{}", h))
            .unwrap_or_default()
    } else {
        security
            .allowed_hosts
            .iter()
            .find(|h| !h.contains("localhost") && !h.contains("127.0.0.1"))
            .map(|h| format!("https://{}", h))
            .unwrap_or_default()
    };

    if og_image.starts_with("http://") || og_image.starts_with("https://") {
        og_image.to_string()
    } else {
        let version = crate::utils::env::css_token();
        if version.is_empty() {
            format!("{}{}", host, og_image)
        } else {
            format!("{}{}?v={}", host, og_image, version)
        }
    }
}
