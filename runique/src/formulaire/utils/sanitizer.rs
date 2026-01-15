use once_cell::sync::Lazy;
use regex::Regex;

/// Liste des balises HTML à supprimer systématiquement car elles n'ont pas leur place
/// dans un contenu textuel sécurisé (XSS)
static DANGEROUS_TAGS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"(?i)<script[^>]*>.*?</script>|<script[^>]*>|</script>|<iframe[^>]*>.*?</iframe>|<iframe[^>]*>|<embed[^>]*>|<object[^>]*>.*?</object>|<meta[^>]*>|<link[^>]*>|<style[^>]*>.*?</style>"#).unwrap()
});

/// Regex pour détecter les attributs d'événements JavaScript
static JS_EVENTS: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"(?i)\bon\w+\s*=\s*(?:'[^']*'|"[^"]*"|[^\s>]+)"#).unwrap());

/// Regex pour détecter le protocole javascript:
static JS_PROTOCOL: Lazy<Regex> = Lazy::new(|| Regex::new(r#"(?i)javascript:\s*"#).unwrap());

/// Regex pour détecter toutes les balises HTML restantes
static ALL_HTML_TAGS: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]*>").unwrap());

/// Sanitise une chaîne de caractères contre les attaques XSS
/// tout en préservant la mise en forme
pub fn auto_sanitize(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }

    // 1. Supprimer les balises les plus dangereuses et leur contenu
    let mut cleaned = DANGEROUS_TAGS.replace_all(input, "").to_string();

    // 2. Supprimer les gestionnaires d'événements JS (ex: onclick=...)
    cleaned = JS_EVENTS.replace_all(&cleaned, "").to_string();

    // 3. Supprimer le protocole javascript:
    cleaned = JS_PROTOCOL.replace_all(&cleaned, "").to_string();

    // 4. Supprimer toutes les balises HTML restantes (strip_tags)
    cleaned = ALL_HTML_TAGS.replace_all(&cleaned, "").to_string();

    // les sauts de ligne et l'indentation des formulaires (textareas).
    cleaned.trim().to_string()
}
/// Vérifie si une clé de formulaire est sensible (mot de passe, token, etc.)
pub fn is_sensitive_field(key: &str) -> bool {
    let key = key.to_lowercase();
    key.contains("password")
        || key.contains("token")
        || key.contains("secret")
        || key.contains("key")
}

/// Vérifie si une valeur contient des éléments suspects
pub fn is_dangerous(value: &str) -> bool {
    DANGEROUS_TAGS.is_match(value)
        || JS_EVENTS.is_match(value)
        || JS_PROTOCOL.is_match(value)
        || ALL_HTML_TAGS.is_match(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xss_protection() {
        let input = r#"<script>alert('xss')</script>Bonjour <img src="x" onerror="alert(1)">, voici du javascript:void(0)"#;
        let output = auto_sanitize(input);
        // Devrait supprimer les balises et les attributs dangereux
        assert!(!output.contains("<script>"));
        assert!(!output.contains("onerror"));
        assert!(!output.contains("javascript:"));
    }

    #[test]
    fn test_preserve_formatting() {
        let input = "Ligne 1\nLigne 2    Espaces";
        let output = auto_sanitize(input);
        // Doit préserver les sauts de ligne et les multiples espaces
        assert_eq!(output, "Ligne 1\nLigne 2    Espaces");
    }
}
