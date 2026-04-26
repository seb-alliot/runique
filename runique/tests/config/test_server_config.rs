// Tests pour ServerConfig

use crate::utils::env::{del_env, set_env};
use runique::config::server::ServerConfig;
use serial_test::serial;

// ── Valeurs par défaut (sans variables d'environnement) ────────────────────────

#[test]
#[serial]
fn test_server_config_default_ip() {
    del_env("IP_SERVER");
    del_env("PORT");
    del_env("SECRET_KEY");
    let config = ServerConfig::from_env();
    assert_eq!(config.ip_server, "127.0.0.1");
}

#[test]
#[serial]
fn test_server_config_default_port() {
    del_env("PORT");
    let config = ServerConfig::from_env();
    assert_eq!(config.port, 3000);
}

#[test]
#[serial]
fn test_server_config_default_secret_key() {
    del_env("SECRET_KEY");
    let config = ServerConfig::from_env();
    assert_eq!(config.secret_key, "default_secret_key");
}

#[test]
#[serial]
fn test_server_config_domain_server_construit_correctement() {
    del_env("IP_SERVER");
    del_env("PORT");
    let config = ServerConfig::from_env();
    assert_eq!(
        config.domain_server,
        format!("{}:{}", config.ip_server, config.port)
    );
}

// ── Lecture depuis variables d'environnement ───────────────────────────────────

#[test]
#[serial]
fn test_server_config_ip_personnalise() {
    set_env("IP_SERVER", "0.0.0.0");
    let config = ServerConfig::from_env();
    assert_eq!(config.ip_server, "0.0.0.0");
    del_env("IP_SERVER");
}

#[test]
#[serial]
fn test_server_config_port_personnalise() {
    set_env("PORT", "8080");
    let config = ServerConfig::from_env();
    assert_eq!(config.port, 8080);
    del_env("PORT");
}

#[test]
#[serial]
fn test_server_config_secret_key_personnalise() {
    set_env("SECRET_KEY", "ma_cle_super_secrete");
    let config = ServerConfig::from_env();
    assert_eq!(config.secret_key, "ma_cle_super_secrete");
    del_env("SECRET_KEY");
}

#[test]
#[serial]
fn test_server_config_domain_server_avec_ip_et_port_personnalises() {
    set_env("IP_SERVER", "10.0.0.1");
    set_env("PORT", "9000");
    let config = ServerConfig::from_env();
    assert_eq!(config.domain_server, "10.0.0.1:9000");
    del_env("IP_SERVER");
    del_env("PORT");
}

#[test]
#[serial]
fn test_server_config_port_invalide_utilise_defaut() {
    set_env("PORT", "pas_un_nombre");
    let config = ServerConfig::from_env();
    assert_eq!(config.port, 3000);
    del_env("PORT");
}

// ── Clone et Default ───────────────────────────────────────────────────────────

#[test]
fn test_server_config_clone() {
    let config = ServerConfig {
        ip_server: "192.168.1.1".to_string(),
        domain_server: "192.168.1.1:3000".to_string(),
        port: 3000,
        secret_key: "secret".to_string(),
    };
    let cloned = config.clone();
    assert_eq!(cloned.ip_server, config.ip_server);
    assert_eq!(cloned.port, config.port);
    assert_eq!(cloned.secret_key, config.secret_key);
}

#[test]
fn test_server_config_default_trait() {
    let config = ServerConfig::default();
    assert!(config.ip_server.is_empty());
    assert_eq!(config.port, 0);
}
