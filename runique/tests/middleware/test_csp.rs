// Tests pour csp middleware

use runique::app::staging::CspConfig;
use runique::middleware::security::csp::SecurityPolicy;

#[test]
fn test_security_policy_default() {
    let policy = SecurityPolicy::default();
    assert!(policy.default_src.contains(&"'self'".to_string()));
    assert!(policy.script_src.contains(&"'self'".to_string()));
    assert!(policy.style_src.contains(&"'self'".to_string()));
    assert!(policy.img_src.contains(&"'self'".to_string()));
    assert!(policy.use_nonce);
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
fn test_csp_config_default_policy() {
    // CspConfig::default() demarre avec SecurityPolicy::default()
    let csp = CspConfig::default();
    assert!(csp.get_policy().default_src.contains(&"'self'".to_string()));
    assert!(!csp.header_security_enabled());
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
    let mut policy = SecurityPolicy {
        use_nonce: true,
        ..Default::default()
    };
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
fn test_csp_config_custom_default_src() {
    let csp = CspConfig::default().default_src(vec!["'self'", "cdn.example.com"]);
    assert!(
        csp.get_policy()
            .default_src
            .contains(&"cdn.example.com".to_string())
    );
}

#[test]
fn test_csp_config_custom_scripts() {
    let csp = CspConfig::default().scripts(vec!["'self'", "cdn.js.com"]);
    assert!(
        csp.get_policy()
            .script_src
            .contains(&"cdn.js.com".to_string())
    );
}

#[test]
fn test_csp_config_nonce_false() {
    let csp = CspConfig::default().with_nonce(false);
    assert!(!csp.get_policy().use_nonce);
}

#[test]
fn test_csp_config_nonce_true_par_defaut() {
    let csp = CspConfig::default();
    assert!(csp.get_policy().use_nonce);
}

#[test]
fn test_csp_config_header_security() {
    let csp = CspConfig::default().with_header_security(true);
    assert!(csp.header_security_enabled());
}

#[test]
fn test_csp_config_upgrade_insecure() {
    let csp = CspConfig::default().with_upgrade_insecure(true);
    assert!(csp.get_policy().upgrade_insecure_requests);
}

#[test]
fn test_csp_config_preset_strict() {
    let csp = CspConfig::default().policy(SecurityPolicy::strict());
    assert!(csp.get_policy().use_nonce);
    assert!(csp.get_policy().upgrade_insecure_requests);
}
