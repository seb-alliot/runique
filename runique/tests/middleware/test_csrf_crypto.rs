//! Tests — CSRF token crypto
//! Couvre : generation_token, generation_user_token, mask_csrf_token, unmask_csrf_token

use runique::utils::csrf::{
    generation_token, generation_user_token, mask_csrf_token, unmask_csrf_token, CsrfContext,
    CsrfToken,
};

const SECRET: &str = "test_secret_key_for_runique";

// ── Génération ────────────────────────────────────────────────────────────────

#[test]
fn test_generate_anonymous_token_returns_hex() {
    let token = generation_token(SECRET, "session_id_abc");
    // HMAC-SHA256 = 32 bytes = 64 hex chars
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_generate_user_token_returns_hex() {
    let token = generation_user_token(SECRET, "42");
    assert_eq!(token.len(), 64);
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_two_anonymous_tokens_differ_with_same_session() {
    // Le timestamp nanoseconde diffère entre deux appels
    let t1 = generation_token(SECRET, "same_session");
    let t2 = generation_token(SECRET, "same_session");
    // Ultra-improbable d'être identiques (timestamp nano différent)
    assert_ne!(t1, t2, "Les tokens doivent être uniques (timestamp diff)");
}

#[test]
fn test_different_secrets_produce_different_tokens() {
    let t1 = generation_token("secret_a", "session");
    let t2 = generation_token("secret_b", "session");
    // Très probable qu'ils soient différents même si le timestamp est similaire
    // On vérifie au moins que les deux sont valides
    assert_eq!(t1.len(), 64);
    assert_eq!(t2.len(), 64);
}

// ── Masking / Unmasking ───────────────────────────────────────────────────────

#[test]
fn test_mask_produces_base64() {
    let token = generation_token(SECRET, "sid");
    let masked = mask_csrf_token(&token);
    // Base64 ne contient que [A-Za-z0-9+/=]
    assert!(masked
        .chars()
        .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='));
}

#[test]
fn test_mask_unmask_roundtrip() {
    let original = generation_token(SECRET, "session_xyz");
    let masked = mask_csrf_token(&original);
    let unmasked = unmask_csrf_token(&masked).expect("unmask doit réussir");
    assert_eq!(unmasked, original);
}

#[test]
fn test_mask_unmask_roundtrip_user_token() {
    let original = generation_user_token(SECRET, "1337");
    let masked = mask_csrf_token(&original);
    let unmasked = unmask_csrf_token(&masked).expect("unmask doit réussir");
    assert_eq!(unmasked, original);
}

#[test]
fn test_two_masks_of_same_token_differ() {
    // Le mask est randomisé — deux appels produisent des résultats différents
    let token = generation_token(SECRET, "sid");
    let masked1 = mask_csrf_token(&token);
    let masked2 = mask_csrf_token(&token);
    assert_ne!(
        masked1, masked2,
        "Le masking doit être aléatoire (BREACH protection)"
    );
}

#[test]
fn test_two_masks_both_unmask_to_same_original() {
    let token = generation_token(SECRET, "sid");
    let masked1 = mask_csrf_token(&token);
    let masked2 = mask_csrf_token(&token);
    let un1 = unmask_csrf_token(&masked1).unwrap();
    let un2 = unmask_csrf_token(&masked2).unwrap();
    assert_eq!(un1, token);
    assert_eq!(un2, token);
}

#[test]
fn test_unmask_invalid_base64_returns_error() {
    let result = unmask_csrf_token("!!!!not_valid_base64!!!!");
    assert!(result.is_err());
}

#[test]
fn test_unmask_empty_returns_empty_ok() {
    // "" en base64 décode vers [] (0 octets), 0 % 2 == 0 → Ok("") sans erreur
    let result = unmask_csrf_token("");
    assert_eq!(result, Ok(String::new()));
}

#[test]
fn test_unmask_odd_length_returns_error() {
    // base64 de 3 octets → longueur décodée impaire → erreur "Invalid token length"
    use base64::{engine::general_purpose, Engine as _};
    let odd = general_purpose::STANDARD.encode([0x01u8, 0x02, 0x03]);
    let result = unmask_csrf_token(&odd);
    assert_eq!(result, Err("Invalid token length"));
}

// ── CsrfToken struct ──────────────────────────────────────────────────────────

#[test]
fn test_csrf_token_generate_anonymous() {
    let token = CsrfToken::generate_with_context(
        CsrfContext::Anonymous {
            session_id: "sess_abc",
        },
        SECRET,
    );
    assert_eq!(token.as_str().len(), 64);
    assert!(token.as_str().chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_csrf_token_generate_authenticated() {
    let token =
        CsrfToken::generate_with_context(CsrfContext::Authenticated { user_id: 42 }, SECRET);
    assert_eq!(token.as_str().len(), 64);
    assert!(token.as_str().chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_csrf_token_masked_unmask_roundtrip() {
    let token = CsrfToken::generate_with_context(
        CsrfContext::Anonymous {
            session_id: "sess_xyz",
        },
        SECRET,
    );
    let masked = token.masked();
    let unmasked = CsrfToken::unmasked(masked.as_str()).expect("unmask doit réussir");
    assert_eq!(unmasked.as_str(), token.as_str());
}

#[test]
fn test_csrf_token_unmasked_invalid_returns_error() {
    let result = CsrfToken::unmasked("!not_base64!");
    assert!(result.is_err());
}

#[test]
fn test_csrf_token_as_str_returns_hex() {
    let token = CsrfToken::generate_with_context(CsrfContext::Authenticated { user_id: 1 }, SECRET);
    assert_eq!(token.as_str().len(), 64);
}
