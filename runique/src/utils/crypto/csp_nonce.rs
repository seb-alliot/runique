//! CSP nonce generation — 16 random bytes encoded in base64, renewed for each request.

/// CSP Nonce Generation
///
/// Utilities to generate CSP (Content Security Policy) nonces
/// for inline scripts and styles.
use rand::RngExt;
/// A base64-encoded, per-request CSP nonce, allowing an inline `<script>`/`<style>`
/// tagged with the matching `nonce="..."` attribute to run under a strict CSP.
#[derive(Debug, Clone)]
pub struct CspNonce(String);

impl CspNonce {
    /// Generates a new nonce from 16 random bytes (128 bits), base64-encoded.
    #[must_use]
    pub fn generate() -> Self {
        let mut rng = rand::rng();
        let mut bytes = [0u8; 16]; // 16 bytes = 128 bits
        rng.fill(&mut bytes);
        let nonce = CspNonce::base64_encode(&bytes);
        CspNonce(nonce)
    }
    /// Returns the encoded nonce value, as used in the `nonce` attribute and the
    /// `Content-Security-Policy` header.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Base64-encodes arbitrary bytes using the standard alphabet (with padding).
    #[must_use]
    pub fn base64_encode(data: &[u8]) -> String {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;
        engine.encode(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nonce_attr() -> String {
        let nonce = CspNonce::generate();
        format!("nonce=\"{}\"", nonce.as_str())
    }

    #[test]
    fn test_nonce_generation() {
        let nonce1 = CspNonce::generate();
        let nonce2 = CspNonce::generate();

        assert!(!nonce1.as_str().is_empty());
        assert!(!nonce2.as_str().is_empty());
        assert_ne!(nonce1.as_str(), nonce2.as_str());
    }

    #[test]
    fn test_nonce_attr() {
        let attr = nonce_attr();
        assert!(attr.starts_with("nonce="));
        assert!(attr.contains('"'));
    }
}
