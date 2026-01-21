/// CSP Nonce Generation
///
/// Utilitaires pour générer des nonces CSP (Content Security Policy)
/// pour les scripts inline et styles.
use rand::Rng;

/// Génère un nonce CSP aléatoire (base64)
///
/// # Exemple
/// ```rust
/// use runique::utils::csp_nonce::generate_csp_nonce;
///
/// let nonce = generate_csp_nonce();
/// assert!(!nonce.is_empty());
/// assert!(nonce.len() >= 16); // Au minimum 16 bytes en base64
/// ```
pub fn generate_csp_nonce() -> String {
    let mut rng = rand::rng();
    let mut bytes = [0u8; 32]; // 32 bytes = 256 bits
    rng.fill(&mut bytes);

    base64_encode(&bytes)
}

/// Encode les bytes en base64
fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    let engine = base64::engine::general_purpose::STANDARD;
    engine.encode(data)
}

/// Crée un attribut `nonce="..."` pour un script
///
/// # Exemple
/// ```rust
/// use runique::utils::csp_nonce::nonce_attr;
///
/// let attr = nonce_attr();
/// // Affiche: nonce="..."
/// println!("<script {}>", attr);
/// ```
pub fn nonce_attr() -> String {
    format!(r#"nonce="{}""#, generate_csp_nonce())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_generation() {
        let nonce1 = generate_csp_nonce();
        let nonce2 = generate_csp_nonce();

        assert!(!nonce1.is_empty());
        assert!(!nonce2.is_empty());
        assert_ne!(nonce1, nonce2); // Doivent être différents
    }

    #[test]
    fn test_nonce_attr() {
        let attr = nonce_attr();
        assert!(attr.starts_with("nonce="));
        assert!(attr.contains('"'));
    }
}
