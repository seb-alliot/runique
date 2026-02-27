// Tests pour SecurityConfig

use runique::config::security::SecurityConfig;
use serial_test::serial;

// ── Valeurs par défaut (sans variables d'environnement) ────────────────────────

#[test]
#[serial]
fn test_security_config_defaults_sanitize_inputs() {
    std::env::remove_var("SANITIZE_INPUTS");
    let config = SecurityConfig::from_env();
    assert!(
        config.sanitize_inputs,
        "sanitize_inputs doit être true par défaut"
    );
}

#[test]
#[serial]
fn test_security_config_defaults_strict_csp() {
    std::env::remove_var("STRICT_CSP");
    let config = SecurityConfig::from_env();
    assert!(config.strict_csp, "strict_csp doit être true par défaut");
}

#[test]
#[serial]
fn test_security_config_defaults_rate_limiting() {
    std::env::remove_var("RATE_LIMITING");
    let config = SecurityConfig::from_env();
    assert!(
        config.rate_limiting,
        "rate_limiting doit être true par défaut"
    );
}

#[test]
#[serial]
fn test_security_config_defaults_enforce_https() {
    std::env::remove_var("ENFORCE_HTTPS");
    let config = SecurityConfig::from_env();
    assert!(
        !config.enforce_https,
        "enforce_https doit être false par défaut"
    );
}

#[test]
#[serial]
fn test_security_config_defaults_allowed_hosts() {
    std::env::remove_var("ALLOWED_HOSTS");
    let config = SecurityConfig::from_env();
    assert!(
        config.allowed_hosts.contains(&"localhost".to_string()),
        "localhost doit être dans allowed_hosts par défaut"
    );
    assert!(
        config.allowed_hosts.contains(&"127.0.0.1".to_string()),
        "127.0.0.1 doit être dans allowed_hosts par défaut"
    );
}

// ── Lecture depuis variables d'environnement ───────────────────────────────────

#[test]
#[serial]
fn test_security_config_sanitize_inputs_false() {
    std::env::set_var("SANITIZE_INPUTS", "false");
    let config = SecurityConfig::from_env();
    assert!(!config.sanitize_inputs);
    std::env::remove_var("SANITIZE_INPUTS");
}

#[test]
#[serial]
fn test_security_config_strict_csp_false() {
    std::env::set_var("STRICT_CSP", "false");
    let config = SecurityConfig::from_env();
    assert!(!config.strict_csp);
    std::env::remove_var("STRICT_CSP");
}

#[test]
#[serial]
fn test_security_config_enforce_https_true() {
    std::env::set_var("ENFORCE_HTTPS", "true");
    let config = SecurityConfig::from_env();
    assert!(config.enforce_https);
    std::env::remove_var("ENFORCE_HTTPS");
}

#[test]
#[serial]
fn test_security_config_rate_limiting_false() {
    std::env::set_var("RATE_LIMITING", "false");
    let config = SecurityConfig::from_env();
    assert!(!config.rate_limiting);
    std::env::remove_var("RATE_LIMITING");
}

#[test]
#[serial]
fn test_security_config_allowed_hosts_personnalises() {
    std::env::set_var("ALLOWED_HOSTS", "example.com, api.example.com");
    let config = SecurityConfig::from_env();
    assert!(config.allowed_hosts.contains(&"example.com".to_string()));
    assert!(config
        .allowed_hosts
        .contains(&"api.example.com".to_string()));
    std::env::remove_var("ALLOWED_HOSTS");
}

#[test]
#[serial]
fn test_security_config_allowed_hosts_un_seul() {
    std::env::set_var("ALLOWED_HOSTS", "monsite.fr");
    let config = SecurityConfig::from_env();
    assert_eq!(config.allowed_hosts.len(), 1);
    assert_eq!(config.allowed_hosts[0], "monsite.fr");
    std::env::remove_var("ALLOWED_HOSTS");
}

// ── Clone et Debug ─────────────────────────────────────────────────────────────

#[test]
fn test_security_config_clone() {
    let config = SecurityConfig {
        sanitize_inputs: true,
        strict_csp: false,
        rate_limiting: true,
        enforce_https: true,
        allowed_hosts: vec!["localhost".to_string()],
    };
    let cloned = config.clone();
    assert_eq!(cloned.sanitize_inputs, config.sanitize_inputs);
    assert_eq!(cloned.strict_csp, config.strict_csp);
    assert_eq!(cloned.enforce_https, config.enforce_https);
    assert_eq!(cloned.allowed_hosts, config.allowed_hosts);
}

#[test]
fn test_security_config_default_trait() {
    let config = SecurityConfig::default();
    // Default via derive : tout à false/empty
    assert!(!config.sanitize_inputs);
    assert!(!config.strict_csp);
    assert!(!config.rate_limiting);
    assert!(!config.enforce_https);
    assert!(config.allowed_hosts.is_empty());
}
