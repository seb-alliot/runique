// Tests pour csp middleware

use runique::middleware::security::csp::SecurityPolicy;

#[test]
fn test_security_policy_default() {
    let policy = SecurityPolicy::default();
    assert!(policy.default_src.contains(&"'self'".to_string()));
    assert!(policy.script_src.contains(&"'self'".to_string()));
    assert!(policy.style_src.contains(&"'self'".to_string()));
    assert!(policy.img_src.contains(&"data:".to_string()));
    assert!(!policy.use_nonce);
}

#[test]
fn test_security_policy_strict() {
    let policy = SecurityPolicy::strict();
    assert!(policy.use_nonce);
    assert_eq!(policy.frame_ancestors, vec!["'none'".to_string()]);
}

#[test]
fn test_security_policy_permissive() {
    let policy = SecurityPolicy::permissive();
    assert!(!policy.use_nonce);
    assert!(policy.script_src.contains(&"'unsafe-eval'".to_string()));
    assert!(policy.img_src.contains(&"https:".to_string()));
}

#[test]
fn test_security_policy_from_env_defaults() {
    // On ne définit pas de variables d'env, donc on doit avoir les valeurs par défaut
    let policy = SecurityPolicy::from_env();
    assert!(policy.default_src.contains(&"'self'".to_string()));
}

#[test]
fn test_to_header_value_basic() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(None);
    assert!(header.contains("default-src"));
    assert!(header.contains("script-src"));
    assert!(header.contains("style-src"));
    assert!(header.contains("img-src"));
}

#[test]
fn test_to_header_value_with_nonce() {
    let mut policy = SecurityPolicy::default();
    policy.use_nonce = true;
    let header = policy.to_header_value(Some("abc123"));
    assert!(header.contains("'nonce-abc123'"));
}
