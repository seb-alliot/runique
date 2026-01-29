use crate::utils::constante::parse::{ALLOWED_ATTRS, ALLOWED_TAGS, RICH_CONTENT_FIELDS};
use ammonia::Builder;
use std::collections::HashSet;

/// Sanitise intelligemment selon le nom du champ
pub fn auto_sanitize_field(field_name: &str, input: &str) -> String {
    if RICH_CONTENT_FIELDS.contains(field_name) {
        auto_sanitize_with_formatting(input)
    } else {
        auto_sanitize(input)
    }
}

/// Mode STRICT : supprime absolument tout HTML
pub fn auto_sanitize(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    Builder::new()
        .tags(HashSet::new()) // aucune balise autorisée
        .clean(input)
        .to_string()
        .trim()
        .to_string()
}

/// Mode PERMISSIF : autorise un sous-ensemble propre de HTML
pub fn auto_sanitize_with_formatting(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    Builder::new()
        .tags(ALLOWED_TAGS.clone())
        .tag_attributes(ALLOWED_ATTRS.clone())
        .link_rel(Some("noopener noreferrer"))
        .clean(input)
        .to_string()
        .trim()
        .to_string()
}

/// Vérifie si une clé de formulaire est sensible
pub fn is_sensitive_field(key: &str) -> bool {
    let key = key.to_lowercase();

    key.contains("password")
        || key.contains("passwd")
        || key.contains("pwd")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("api_key")
        || key.contains("key")
}

/// Détection heuristique de contenu dangereux
/// (utile pour logger / refuser certaines entrées)
pub fn is_dangerous(value: &str) -> bool {
    let lower = value.to_lowercase();

    lower.contains("<script")
        || lower.contains("javascript:")
        || lower.contains("onerror=")
        || lower.contains("onload=")
        || lower.contains("onclick=")
        || lower.contains("<iframe")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xss_protection_strict() {
        let input = r#"<script>alert('xss')</script>Bonjour <img src="x" onerror="alert(1)"> javascript:void(0)"#;
        let output = auto_sanitize(input);

        assert!(!output.contains("<script"));
        assert!(!output.contains("onerror"));
        assert!(!output.contains("javascript:"));
        assert!(output.contains("Bonjour"));
    }

    #[test]
    fn test_preserve_formatting() {
        let input = "Ligne 1\nLigne 2    Espaces";
        let output = auto_sanitize(input);

        assert_eq!(output, input);
    }

    #[test]
    fn test_rich_content_allows_tags() {
        let input = "<p>Hello <strong>world</strong></p><script>alert(1)</script>";
        let output = auto_sanitize_with_formatting(input);

        assert!(output.contains("<p>"));
        assert!(output.contains("<strong>"));
        assert!(!output.contains("<script"));
    }
}
