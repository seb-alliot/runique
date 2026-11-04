//! Encryption of the session payload kept in `eihwaz_sessions`: reading the
//! database (a backup, an injection) doesn't give the content of the sessions,
//! and a payload altered there is refused.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use hmac::{Hmac, Mac};
use rand::RngExt;
use sha2::Sha256;

/// Marks the format, so a later one can be told apart. A value without it (a
/// payload stored in clear before 4.0) is refused: accepting it would let
/// whoever can write to the database forge a session.
const FORMAT: &str = "v1:";
const NONCE_LEN: usize = 24;

/// The key of the session payloads, derived from `SECRET_KEY`.
#[derive(Clone)]
pub struct SessionKey([u8; 32]);

/// Written by hand: `RuniqueSessionStore` derives `Debug`, the key must not follow.
impl std::fmt::Debug for SessionKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionKey(..)")
    }
}

impl SessionKey {
    /// The label keeps this key apart from what `SECRET_KEY` signs elsewhere (CSRF).
    #[must_use]
    pub fn derive(secret_key: &str) -> Self {
        let mut mac = Hmac::<Sha256>::new_from_slice(secret_key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(b"runique.session.data.v1");
        let mut key = [0u8; 32];
        key.copy_from_slice(&mac.finalize().into_bytes());
        Self(key)
    }

    fn cipher(&self) -> XChaCha20Poly1305 {
        XChaCha20Poly1305::new(&self.0.into())
    }

    /// Encrypts `plain`, tied to `bound_to`: the result only opens with that
    /// same value, so a payload copied onto another row is refused. XChaCha20's
    /// 192-bit nonce can be drawn at random for each write under a long-lived key.
    #[must_use]
    pub fn seal(&self, plain: &str, bound_to: &str) -> Option<String> {
        let mut nonce = [0u8; NONCE_LEN];
        rand::rng().fill(&mut nonce);
        let encrypted = self
            .cipher()
            .encrypt(
                &XNonce::from(nonce),
                Payload {
                    msg: plain.as_bytes(),
                    aad: bound_to.as_bytes(),
                },
            )
            .ok()?;
        let mut raw = nonce.to_vec();
        raw.extend(encrypted);
        Some(format!("{FORMAT}{}", STANDARD.encode(raw)))
    }

    /// The text [`seal`](Self::seal) was given. `None` for anything else: another
    /// key, another `bound_to`, an altered value, a value in clear.
    #[must_use]
    pub fn open(&self, sealed: &str, bound_to: &str) -> Option<String> {
        let raw = STANDARD.decode(sealed.strip_prefix(FORMAT)?).ok()?;
        let (nonce, encrypted) = raw.split_first_chunk::<NONCE_LEN>()?;
        let plain = self
            .cipher()
            .decrypt(
                &XNonce::from(*nonce),
                Payload {
                    msg: encrypted,
                    aad: bound_to.as_bytes(),
                },
            )
            .ok()?;
        String::from_utf8(plain).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::SessionKey;

    #[test]
    fn debug_never_shows_the_key() {
        let key = SessionKey([0xAB; 32]);
        let shown = format!("{key:?}");
        assert_eq!(shown, "SessionKey(..)");
        let raw = format!("{:?}", key.0);
        assert!(!shown.contains(&raw));
    }
}
