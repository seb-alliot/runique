// Tests pour StaticConfig

use runique::config::static_files::StaticConfig;
use serial_test::serial;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut (sans variables d'environnement)
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_static_config_default_static_url() {
    unsafe {
    std::env::remove_var("STATIC_URL");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_url, "/static");
}

#[test]
#[serial]
fn test_static_config_default_media_url() {
    unsafe {
        std::env::remove_var("MEDIA_URL");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_url, "/media");
}

#[test]
#[serial]
fn test_static_config_default_staticfiles_dirs() {
    unsafe {
        std::env::remove_var("STATICFILES_DIRS");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.staticfiles_dirs, "static");
}

#[test]
#[serial]
fn test_static_config_default_media_root() {
    unsafe {
        std::env::remove_var("MEDIA_ROOT");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_root, "media");
}

#[test]
#[serial]
fn test_static_config_default_staticfiles() {
    unsafe {
        std::env::remove_var("STATICFILES");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.staticfiles, "default_storage");
}

#[test]
#[serial]
fn test_static_config_default_templates_dir() {
    unsafe {
        std::env::remove_var("TEMPLATES_DIR");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.templates_dir, vec!["templates".to_string()]);
}

#[test]
#[serial]
fn test_static_config_default_static_runique_url() {
    unsafe {
        std::env::remove_var("STATIC_RUNIQUE_URL");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_runique_url, "/runique/static");
}

#[test]
#[serial]
fn test_static_config_default_media_runique_url() {
    unsafe {
        std::env::remove_var("MEDIA_RUNIQUE_URL");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_runique_url, "/runique/media");
}

#[test]
#[serial]
fn test_static_config_default_base_dir() {
    unsafe {
        std::env::remove_var("BASE_DIR");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.base_dir, ".");
}

// ═══════════════════════════════════════════════════════════════
// Lecture depuis variables d'environnement
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_static_config_static_url_personnalise() {
    unsafe {
        std::env::set_var("STATIC_URL", "/assets");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_url, "/assets");
    unsafe {
        std::env::remove_var("STATIC_URL");
    }
}

#[test]
#[serial]
fn test_static_config_media_url_personnalise() {
    unsafe {
        std::env::set_var("MEDIA_URL", "/uploads");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_url, "/uploads");
    unsafe {
        std::env::remove_var("MEDIA_URL");
    }
}

#[test]
#[serial]
fn test_static_config_templates_dir_multiples() {
    unsafe {
        std::env::set_var("TEMPLATES_DIR", "templates/front, templates/admin");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.templates_dir.len(), 2);
    assert_eq!(cfg.templates_dir[0], "templates/front");
    assert_eq!(cfg.templates_dir[1], "templates/admin");
    unsafe {
        std::env::remove_var("TEMPLATES_DIR");
    }
}

#[test]
#[serial]
fn test_static_config_base_dir_personnalise() {
    unsafe {
        std::env::set_var("BASE_DIR", "/app");
    }
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.base_dir, "/app");
    unsafe {
        std::env::remove_var("BASE_DIR");
    }
}

// ═══════════════════════════════════════════════════════════════
// Default derive
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_static_config_default_derive_vide() {
    let cfg = StaticConfig::default();
    assert!(cfg.static_url.is_empty());
    assert!(cfg.media_url.is_empty());
    assert!(cfg.templates_dir.is_empty());
    assert!(cfg.base_dir.is_empty());
}
