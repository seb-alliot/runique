// Tests pour SecurityConfig

use crate::utils::env::{del_env, set_env};
use runique::config::security::SecurityConfig;
use serial_test::serial;

// ── Valeurs par défaut (sans variables d'environnement) ────────────────────────

#[test]
#[serial]
fn test_security_config_defaults_rate_limiting() {
    del_env("RATE_LIMITING");
    let config = SecurityConfig::from_env();
    assert!(
        config.rate_limiting,
        "rate_limiting doit être true par défaut"
    );
}

#[test]
#[serial]
fn test_security_config_defaults_enforce_https() {
    del_env("ENFORCE_HTTPS");
    let config = SecurityConfig::from_env();
    assert!(
        !config.enforce_https,
        "enforce_https doit être false par défaut"
    );
}

#[test]
#[serial]
fn test_security_config_defaults_allowed_hosts() {
    del_env("ALLOWED_HOSTS");
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
fn test_security_config_enforce_https_true() {
    set_env("ENFORCE_HTTPS", "true");
    let config = SecurityConfig::from_env();
    assert!(config.enforce_https);
    del_env("ENFORCE_HTTPS");
}

// Regression: `str::parse::<bool>()` only took `true`, so `True` fell back to
// the default `false` and HTTPS wasn't enforced, without a word.
#[test]
#[serial]
fn test_security_config_enforce_https_whatever_the_case() {
    for value in ["True", "TRUE", "1", "yes", "On"] {
        set_env("ENFORCE_HTTPS", value);
        assert!(
            SecurityConfig::from_env().enforce_https,
            "ENFORCE_HTTPS={value} must enforce HTTPS"
        );
    }
    del_env("ENFORCE_HTTPS");
}

#[test]
#[serial]
fn test_security_config_rate_limiting_false() {
    set_env("RATE_LIMITING", "false");
    let config = SecurityConfig::from_env();
    assert!(!config.rate_limiting);
    del_env("RATE_LIMITING");
}

#[test]
#[serial]
fn test_security_config_allowed_hosts_personnalises() {
    set_env("ALLOWED_HOSTS", "example.com, api.example.com");
    let config = SecurityConfig::from_env();
    assert!(config.allowed_hosts.contains(&"example.com".to_string()));
    assert!(
        config
            .allowed_hosts
            .contains(&"api.example.com".to_string())
    );
    del_env("ALLOWED_HOSTS");
}

#[test]
#[serial]
fn test_security_config_allowed_hosts_un_seul() {
    set_env("ALLOWED_HOSTS", "monsite.fr");
    let config = SecurityConfig::from_env();
    assert_eq!(config.allowed_hosts.len(), 1);
    assert_eq!(config.allowed_hosts[0], "monsite.fr");
    del_env("ALLOWED_HOSTS");
}

// ── Clone et Debug ─────────────────────────────────────────────────────────────

#[test]
fn test_security_config_clone() {
    let config = SecurityConfig {
        rate_limiting: true,
        enforce_https: true,
        allowed_hosts: vec!["localhost".to_string()],
        acme_enabled: false,
        acme_domain: None,
        acme_email: None,
        acme_certs_dir: "./certs".to_string(),
        hsts_max_age: 31_536_000,
        hsts_include_subdomains: true,
        hsts_preload: false,
    };
    let cloned = config.clone();
    assert_eq!(cloned.enforce_https, config.enforce_https);
    assert_eq!(cloned.allowed_hosts, config.allowed_hosts);
}

#[test]
fn test_security_config_default_trait() {
    let config = SecurityConfig::default();
    // Default via derive : tout à false/empty
    assert!(!config.rate_limiting);
    assert!(!config.enforce_https);
    assert!(config.allowed_hosts.is_empty());
}

// Empty ACME values mean "not set" — from cargo-mutants survivors (2026-10-02).
#[test]
#[serial]
fn test_security_config_empty_acme_values_are_not_set() {
    set_env("ACME_DOMAIN", "");
    set_env("ACME_EMAIL", "");
    set_env("ACME_CERTS_DIR", "");
    let config = SecurityConfig::from_env();
    assert_eq!(config.acme_domain, None);
    assert_eq!(config.acme_email, None);
    assert_eq!(config.acme_certs_dir, "./certs");

    set_env("ACME_DOMAIN", "example.com");
    set_env("ACME_EMAIL", "ops@example.com");
    set_env("ACME_CERTS_DIR", "/etc/certs");
    let config = SecurityConfig::from_env();
    assert_eq!(config.acme_domain.as_deref(), Some("example.com"));
    assert_eq!(config.acme_email.as_deref(), Some("ops@example.com"));
    assert_eq!(config.acme_certs_dir, "/etc/certs");
    del_env("ACME_DOMAIN");
    del_env("ACME_EMAIL");
    del_env("ACME_CERTS_DIR");
}
