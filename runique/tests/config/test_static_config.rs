// Tests pour StaticConfig

use crate::utils::env::{del_env, restore_media_root, set_env};
use runique::config::static_files::StaticConfig;
use serial_test::serial;

// ═══════════════════════════════════════════════════════════════
// Valeurs par défaut (sans variables d'environnement)
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_static_config_default_static_url() {
    del_env("STATIC_URL");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_url, "/static");
}

#[test]
#[serial]
fn test_static_config_default_media_url() {
    del_env("MEDIA_URL");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_url, "/media");
}

#[test]
#[serial]
fn test_static_config_default_staticfiles_dirs() {
    del_env("STATICFILES_DIRS");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.staticfiles_dirs, "static");
}

#[test]
#[serial]
fn test_static_config_default_media_root() {
    del_env("MEDIA_ROOT");
    del_env("BASE_DIR");
    let cfg = StaticConfig::from_env();
    restore_media_root();
    assert!(
        cfg.media_root.ends_with("/media") || cfg.media_root.ends_with("\\media"),
        "expected media_root to end with /media or \\media, got: {}",
        cfg.media_root
    );
}

#[test]
#[serial]
fn test_static_config_default_staticfiles() {
    del_env("STATICFILES");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.staticfiles, "default_storage");
}

#[test]
#[serial]
fn test_static_config_default_templates_dir() {
    del_env("TEMPLATES_DIR");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.templates_dir, vec!["templates".to_string()]);
}

#[test]
#[serial]
fn test_static_config_default_static_runique_url() {
    del_env("STATIC_RUNIQUE_URL");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_runique_url, "/runique/static");
}

#[test]
#[serial]
fn test_static_config_default_media_runique_url() {
    del_env("MEDIA_RUNIQUE_URL");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_runique, "/runique/media");
}

#[test]
#[serial]
fn test_static_config_default_base_dir() {
    del_env("BASE_DIR");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.base_dir, ".");
}

// ═══════════════════════════════════════════════════════════════
// Lecture depuis variables d'environnement
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_static_config_static_url_personnalise() {
    set_env("STATIC_URL", "/assets");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.static_url, "/assets");
    del_env("STATIC_URL");
}

#[test]
#[serial]
fn test_static_config_media_url_personnalise() {
    set_env("MEDIA_URL", "/uploads");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.media_url, "/uploads");
    del_env("MEDIA_URL");
}

#[test]
#[serial]
fn test_static_config_templates_dir_multiples() {
    set_env("TEMPLATES_DIR", "templates/front, templates/admin");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.templates_dir.len(), 2);
    assert_eq!(cfg.templates_dir[0], "templates/front");
    assert_eq!(cfg.templates_dir[1], "templates/admin");
    del_env("TEMPLATES_DIR");
}

#[test]
#[serial]
fn test_static_config_base_dir_personnalise() {
    set_env("BASE_DIR", "/app");
    let cfg = StaticConfig::from_env();
    assert_eq!(cfg.base_dir, "/app");
    del_env("BASE_DIR");
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

// The body limit has its own key. Unset, it is 100 MB, or the per-file limit
// when that is larger: one file at the per-file limit always fits.
#[test]
#[serial]
fn the_body_limit_defaults_to_100_or_the_per_file_limit() {
    del_env("RUNIQUE_MAX_UPLOAD_MB");
    del_env("RUNIQUE_MAX_BODY_MB");
    let unset = StaticConfig::from_env();
    assert_eq!((unset.max_upload_mb, unset.max_body_mb), (100, 100));

    set_env("RUNIQUE_MAX_UPLOAD_MB", "500");
    let big_file = StaticConfig::from_env();
    assert_eq!(
        big_file.max_body_mb, 500,
        "follows a per-file limit above 100"
    );

    set_env("RUNIQUE_MAX_UPLOAD_MB", "10");
    let small_file = StaticConfig::from_env();
    assert_eq!(small_file.max_body_mb, 100, "stays at 100 under it");

    set_env("RUNIQUE_MAX_BODY_MB", "30");
    let both = StaticConfig::from_env();

    // A body set under the per-file limit is raised to it, never left below.
    set_env("RUNIQUE_MAX_UPLOAD_MB", "200");
    set_env("RUNIQUE_MAX_BODY_MB", "50");
    let body_below = StaticConfig::from_env();
    del_env("RUNIQUE_MAX_UPLOAD_MB");
    del_env("RUNIQUE_MAX_BODY_MB");
    assert_eq!(
        (both.max_upload_mb, both.max_body_mb),
        (10, 30),
        "each key its own value"
    );
    assert_eq!(body_below.max_body_mb, 200, "raised to the per-file limit");
}

// A value that isn't a number is ignored: the default applies, and the body
// still never goes under the per-file limit.
#[test]
#[serial]
fn a_non_numeric_upload_limit_falls_back_to_the_default() {
    set_env("RUNIQUE_MAX_UPLOAD_MB", "lots");
    set_env("RUNIQUE_MAX_BODY_MB", "-5");
    let both_bad = StaticConfig::from_env();

    set_env("RUNIQUE_MAX_UPLOAD_MB", "300");
    set_env("RUNIQUE_MAX_BODY_MB", "abc");
    let bad_body = StaticConfig::from_env();
    del_env("RUNIQUE_MAX_UPLOAD_MB");
    del_env("RUNIQUE_MAX_BODY_MB");

    assert_eq!((both_bad.max_upload_mb, both_bad.max_body_mb), (100, 100));
    assert_eq!(
        (bad_body.max_upload_mb, bad_body.max_body_mb),
        (300, 300),
        "body default raised to the per-file limit"
    );
}

// A config built by `Default` (tests, hand-built engines) carries the same
// limits as an unset `.env`, never 0 MB: file fields are held to them.
#[test]
fn the_default_config_carries_the_documented_upload_limits() {
    let cfg = StaticConfig::default();
    assert_eq!(
        (cfg.max_upload_mb, cfg.max_text_field_kb, cfg.max_body_mb),
        (100, 1024, 100)
    );
}
