// Tests pour csp middleware

use runique::middleware::security::csp::SecurityPolicy;
use serial_test::serial;

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

#[test]
fn test_to_header_value_contains_all_directives() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(None);
    assert!(header.contains("font-src"));
    assert!(header.contains("connect-src"));
    assert!(header.contains("frame-ancestors"));
    assert!(header.contains("base-uri"));
    assert!(header.contains("form-action"));
}

#[test]
fn test_to_header_value_removes_unsafe_inline_with_nonce() {
    let policy = SecurityPolicy::default();
    let header = policy.to_header_value(Some("mynonce"));
    // Le nonce est injecté et 'unsafe-inline' est retiré
    assert!(header.contains("'nonce-mynonce'"));
    assert!(!header.contains("'unsafe-inline'"));
}

#[test]
fn test_strict_no_unsafe_inline_in_script() {
    let policy = SecurityPolicy::strict();
    let header = policy.to_header_value(Some("nonce_strict"));
    assert!(!header.contains("'unsafe-inline'"));
    assert!(header.contains("'nonce-nonce_strict'"));
}

#[test]
fn test_permissive_frame_ancestors_self() {
    let policy = SecurityPolicy::permissive();
    assert_eq!(policy.frame_ancestors, vec!["'self'".to_string()]);
    let header = policy.to_header_value(None);
    assert!(header.contains("frame-ancestors 'self'"));
}

#[test]
#[serial]
fn test_from_env_custom_default_src() {
    std::env::set_var("RUNIQUE_POLICY_CSP_DEFAULT", "'self', cdn.example.com");
    let policy = SecurityPolicy::from_env();
    assert!(policy.default_src.contains(&"cdn.example.com".to_string()));
    std::env::remove_var("RUNIQUE_POLICY_CSP_DEFAULT");
}

#[test]
#[serial]
fn test_from_env_custom_scripts() {
    std::env::set_var("RUNIQUE_POLICY_CSP_SCRIPTS", "'self', cdn.js.com");
    let policy = SecurityPolicy::from_env();
    assert!(policy.script_src.contains(&"cdn.js.com".to_string()));
    std::env::remove_var("RUNIQUE_POLICY_CSP_SCRIPTS");
}

#[test]
#[serial]
fn test_from_env_nonce_false() {
    std::env::set_var("RUNIQUE_POLICY_CSP_STRICT_NONCE", "false");
    let policy = SecurityPolicy::from_env();
    assert!(!policy.use_nonce);
    std::env::remove_var("RUNIQUE_POLICY_CSP_STRICT_NONCE");
}

#[test]
#[serial]
fn test_from_env_nonce_true_by_default() {
    std::env::remove_var("RUNIQUE_POLICY_CSP_STRICT_NONCE");
    let policy = SecurityPolicy::from_env();
    assert!(policy.use_nonce);
}
