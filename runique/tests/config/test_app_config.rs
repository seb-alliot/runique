// Tests pour RuniqueConfig (config agrégée)

use runique::config::app::RuniqueConfig;
use serial_test::serial;

// ═══════════════════════════════════════════════════════════════
// Default derive
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_runique_config_default_debug_false() {
    let cfg = RuniqueConfig::default();
    assert!(!cfg.debug, "debug doit être false via Default::default()");
}

#[test]
fn test_runique_config_default_base_dir_vide() {
    let cfg = RuniqueConfig::default();
    assert!(cfg.base_dir.is_empty());
}

#[test]
fn test_runique_config_clone() {
    let mut cfg = runique::config::RuniqueConfig {
        debug: true,
        base_dir: "/app".to_string(),
        ..Default::default()
    };
    cfg.debug = true;
    cfg.base_dir = "/app".to_string();
    let cloned = cfg.clone();
    assert_eq!(cloned.debug, cfg.debug);
    assert_eq!(cloned.base_dir, cfg.base_dir);
}

// ═══════════════════════════════════════════════════════════════
// from_env — champs directs
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_runique_config_from_env_base_dir_defaut() {
    unsafe {
        std::env::remove_var("BASE_DIR");
    }
    let cfg = RuniqueConfig::from_env();
    assert_eq!(cfg.base_dir, ".");
}

#[test]
#[serial]
fn test_runique_config_from_env_base_dir_personnalise() {
    unsafe {
        std::env::set_var("BASE_DIR", "/srv/app");
    }
    let cfg = RuniqueConfig::from_env();
    assert_eq!(cfg.base_dir, "/srv/app");
    unsafe {
        std::env::remove_var("BASE_DIR");
    }
}

#[test]
#[serial]
fn test_runique_config_from_env_debug_true() {
    unsafe {
        std::env::set_var("DEBUG", "true");
    }
    let cfg = RuniqueConfig::from_env();
    assert!(cfg.debug);
    unsafe {
        std::env::remove_var("DEBUG");
    }
}

#[test]
#[serial]
fn test_runique_config_from_env_debug_false() {
    unsafe {
        std::env::set_var("DEBUG", "false");
    }
    let cfg = RuniqueConfig::from_env();
    assert!(!cfg.debug);
    unsafe {
        std::env::remove_var("DEBUG");
    }
}

#[test]
#[serial]
fn test_runique_config_from_env_debug_invalide_utilise_assertions() {
    unsafe {
        std::env::set_var("DEBUG", "pas_un_bool");
    }
    let cfg = RuniqueConfig::from_env();
    // Valeur invalide → retombe sur cfg!(debug_assertions)
    // En mode test, debug_assertions est true
    assert_eq!(cfg.debug, cfg!(debug_assertions));
    unsafe {
        std::env::remove_var("DEBUG");
    }
}

// ═══════════════════════════════════════════════════════════════
// from_env — sous-configs agrégées
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_runique_config_from_env_contient_server_config() {
    unsafe {
        std::env::remove_var("IP_SERVER");
        std::env::remove_var("PORT");
    }
    let cfg = RuniqueConfig::from_env();
    // ServerConfig::from_env() donne ip="127.0.0.1" par défaut
    assert_eq!(cfg.server.ip_server, "127.0.0.1");
}

#[test]
#[serial]
fn test_runique_config_from_env_contient_security_config() {
    unsafe {
        std::env::remove_var("SANITIZE_INPUTS");
    }
    let cfg = RuniqueConfig::from_env();
    assert!(cfg.security.sanitize_inputs);
}

#[test]
#[serial]
fn test_runique_config_from_env_contient_static_config() {
    unsafe {
        std::env::remove_var("STATIC_URL");
    }
    let cfg = RuniqueConfig::from_env();
    assert_eq!(cfg.static_files.static_url, "/static");
}
