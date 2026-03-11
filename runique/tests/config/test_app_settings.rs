// Tests pour AppSettings et AutoFieldType

use runique::config::settings::AppSettings;
use runique::utils::config::AutoFieldType;
use serial_test::serial;
use crate::utils::env::{set_env, del_env};

// ═══════════════════════════════════════════════════════════════
// AutoFieldType — parse, rust_type, default, from_env
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_autofield_parse_big() {
    let af = AutoFieldType::parse("runique.db.models.BigAutoField");
    assert!(matches!(af, AutoFieldType::BigAutoField));
}

#[test]
fn test_autofield_parse_defaut_pour_inconnu() {
    let af = AutoFieldType::parse("n_importe_quoi");
    assert!(matches!(af, AutoFieldType::AutoField));
}

#[test]
fn test_autofield_parse_defaut_pour_vide() {
    let af = AutoFieldType::parse("");
    assert!(matches!(af, AutoFieldType::AutoField));
}

#[test]
fn test_autofield_rust_type_auto() {
    assert_eq!(AutoFieldType::AutoField.rust_type(), "i32");
}

#[test]
fn test_autofield_rust_type_big() {
    assert_eq!(AutoFieldType::BigAutoField.rust_type(), "i64");
}

#[test]
fn test_autofield_default_est_autofield() {
    assert!(matches!(AutoFieldType::default(), AutoFieldType::AutoField));
}

#[test]
#[serial]
fn test_autofield_from_env_defaut() {
    del_env("DEFAULT_AUTO_FIELD");
    assert!(matches!(
        AutoFieldType::from_env(),
        AutoFieldType::AutoField
    ));
}

#[test]
#[serial]
fn test_autofield_from_env_big() {
    set_env("DEFAULT_AUTO_FIELD", "runique.db.models.BigAutoField");
    assert!(matches!(
        AutoFieldType::from_env(),
        AutoFieldType::BigAutoField
    ));
    del_env("DEFAULT_AUTO_FIELD");
}

// ═══════════════════════════════════════════════════════════════
// AppSettings — valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_app_settings_default_language_code() {
    del_env("LANGUAGE_APP");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.language_code, "en-us");
}

#[test]
#[serial]
fn test_app_settings_default_time_zone() {
    del_env("TIME_ZONE");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.time_zone, "UTC");
}

#[test]
#[serial]
fn test_app_settings_default_use_tz() {
    let cfg = AppSettings::from_env();
    assert!(cfg.use_tz);
}

#[test]
#[serial]
fn test_app_settings_default_installed_apps_vide() {
    let cfg = AppSettings::from_env();
    assert!(cfg.installed_apps.is_empty());
}

#[test]
#[serial]
fn test_app_settings_default_middleware_vide() {
    let cfg = AppSettings::from_env();
    assert!(cfg.middleware.is_empty());
}

#[test]
#[serial]
fn test_app_settings_default_redirect_anonymous() {
    del_env("REDIRECT_ANONYMOUS");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.redirect_anonymous, "/");
}

#[test]
#[serial]
fn test_app_settings_default_logging_required() {
    del_env("LOGGING_URL");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.logging_required, "/");
}

#[test]
#[serial]
fn test_app_settings_default_user_connected() {
    del_env("USER_CONNECTED_URL");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.user_connected, "/");
}

#[test]
#[serial]
fn test_app_settings_default_root_urlconf() {
    del_env("PROJECT_NAME");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.root_urlconf, "myproject.urls");
}

// ═══════════════════════════════════════════════════════════════
// AppSettings — lecture depuis variables d'environnement
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_app_settings_language_code_personnalise() {
    set_env("LANGUAGE_APP", "fr-fr");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.language_code, "fr-fr");
    del_env("LANGUAGE_APP");
}

#[test]
#[serial]
fn test_app_settings_time_zone_personnalise() {
    set_env("TIME_ZONE", "Europe/Paris");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.time_zone, "Europe/Paris");
    del_env("TIME_ZONE");
}

#[test]
#[serial]
fn test_app_settings_project_name_modifie_urlconf() {
    set_env("PROJECT_NAME", "monprojet");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.root_urlconf, "monprojet.urls");
    del_env("PROJECT_NAME");
}

#[test]
#[serial]
fn test_app_settings_redirect_anonymous_personnalise() {
    set_env("REDIRECT_ANONYMOUS", "/login");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.redirect_anonymous, "/login");
    del_env("REDIRECT_ANONYMOUS");
}
