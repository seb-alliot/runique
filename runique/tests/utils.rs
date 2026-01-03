use runique::utils::generate_token;

#[test]
fn test_generate_token() {
    let secret_key = "test_secret_key";
    let session_id = "test_session_123";

    let token1 = generate_token(secret_key, session_id);
    let token2 = generate_token(secret_key, session_id);

    // Les tokens devraient être différents (car ils incluent un timestamp)
    assert_ne!(token1, token2);

    // Mais ils devraient avoir la même longueur (format hex)
    assert_eq!(token1.len(), token2.len());
    assert!(!token1.is_empty());
}

#[test]
fn test_generate_token_different_secrets() {
    let secret1 = "secret1";
    let secret2 = "secret2";
    let session_id = "same_session";

    let token1 = generate_token(secret1, session_id);
    let token2 = generate_token(secret2, session_id);

    // Avec des secrets différents, les tokens devraient être différents
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_token_different_sessions() {
    let secret_key = "same_secret";
    let session1 = "session1";
    let session2 = "session2";

    let token1 = generate_token(secret_key, session1);
    let token2 = generate_token(secret_key, session2);

    // Avec des sessions différentes, les tokens devraient être différents
    assert_ne!(token1, token2);
}

#[test]
fn test_generate_token_format() {
    let secret_key = "test_secret";
    let session_id = "test_session";

    let token = generate_token(secret_key, session_id);

    // Le token devrait être en hex (seulement des caractères 0-9, a-f)
    assert!(token.chars().all(|c| c.is_ascii_hexdigit()));

    // Devrait avoir une longueur raisonnable (64 caractères pour SHA256 en hex)
    assert_eq!(token.len(), 64);
}

#[test]
fn test_generate_token_consistency() {
    let secret_key = "consistent_secret";
    let session_id = "consistent_session";

    // Même si les tokens sont différents à cause du timestamp,
    // la structure devrait être cohérente
    let tokens: Vec<String> = (0..10)
        .map(|_| generate_token(secret_key, session_id))
        .collect();

    // Tous les tokens devraient avoir la même longueur
    let lengths: Vec<usize> = tokens.iter().map(|t| t.len()).collect();
    assert!(lengths.iter().all(|&len| len == lengths[0]));

    // Tous devraient être en hex
    assert!(
        tokens
            .iter()
            .all(|t| t.chars().all(|c| c.is_ascii_hexdigit()))
    );
}
