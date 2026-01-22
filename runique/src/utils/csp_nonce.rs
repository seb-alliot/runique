/// CSP Nonce Generation
///
/// Utilitaires pour générer des nonces CSP (Content Security Policy)
/// pour les scripts inline et styles.
use rand::Rng;
#[derive(Debug, Clone)]
pub struct CspNonce(String);

impl CspNonce{
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 16]; // 16 bytes = 128 bits
        rng.fill(&mut bytes);
        let nonce = CspNonce::base64_encode(&bytes);
        CspNonce(nonce)
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn base64_encode(data: &[u8]) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        engine.encode(data)
    }
}


// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_nonce_generation() {
//         let nonce1 = generate_csp_nonce();
//         let nonce2 = generate_csp_nonce();

//         assert!(!nonce1.is_empty());
//         assert!(!nonce2.is_empty());
//         assert_ne!(nonce1, nonce2); // Doivent être différents
//     }

//     #[test]
//     fn test_nonce_attr() {
//         let attr = nonce_attr();
//         assert!(attr.starts_with("nonce="));
//         assert!(attr.contains('"'));
//     }
// }
