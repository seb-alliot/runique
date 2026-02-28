// Tests pour AppSettings et AutoFieldType

use runique::config::settings::AppSettings;
use runique::utils::config::AutoFieldType;
use serial_test::serial;

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
    std::env::remove_var("DEFAULT_AUTO_FIELD");
    assert!(matches!(
        AutoFieldType::from_env(),
        AutoFieldType::AutoField
    ));
}

#[test]
#[serial]
fn test_autofield_from_env_big() {
    std::env::set_var("DEFAULT_AUTO_FIELD", "runique.db.models.BigAutoField");
    assert!(matches!(
        AutoFieldType::from_env(),
        AutoFieldType::BigAutoField
    ));
    std::env::remove_var("DEFAULT_AUTO_FIELD");
}

// ═══════════════════════════════════════════════════════════════
// AppSettings — valeurs par défaut
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_app_settings_default_language_code() {
    std::env::remove_var("LANGUAGE_APP");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.language_code, "en-us");
}

#[test]
#[serial]
fn test_app_settings_default_time_zone() {
    std::env::remove_var("TIME_ZONE");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.time_zone, "UTC");
}

#[test]
#[serial]
fn test_app_settings_default_use_i18n() {
    let cfg = AppSettings::from_env();
    assert!(cfg.use_i18n);
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
    std::env::remove_var("REDIRECT_ANONYMOUS");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.redirect_anonymous, "/");
}

#[test]
#[serial]
fn test_app_settings_default_logging_required() {
    std::env::remove_var("LOGGING_URL");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.logging_required, "/");
}

#[test]
#[serial]
fn test_app_settings_default_user_connected() {
    std::env::remove_var("USER_CONNECTED_URL");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.user_connected, "/");
}

#[test]
#[serial]
fn test_app_settings_default_root_urlconf() {
    std::env::remove_var("PROJECT_NAME");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.root_urlconf, "myproject.urls");
}

// ═══════════════════════════════════════════════════════════════
// AppSettings — lecture depuis variables d'environnement
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_app_settings_language_code_personnalise() {
    std::env::set_var("LANGUAGE_APP", "fr-fr");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.language_code, "fr-fr");
    std::env::remove_var("LANGUAGE_APP");
}

#[test]
#[serial]
fn test_app_settings_time_zone_personnalise() {
    std::env::set_var("TIME_ZONE", "Europe/Paris");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.time_zone, "Europe/Paris");
    std::env::remove_var("TIME_ZONE");
}

#[test]
#[serial]
fn test_app_settings_project_name_modifie_urlconf() {
    std::env::set_var("PROJECT_NAME", "monprojet");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.root_urlconf, "monprojet.urls");
    std::env::remove_var("PROJECT_NAME");
}

#[test]
#[serial]
fn test_app_settings_redirect_anonymous_personnalise() {
    std::env::set_var("REDIRECT_ANONYMOUS", "/login");
    let cfg = AppSettings::from_env();
    assert_eq!(cfg.redirect_anonymous, "/login");
    std::env::remove_var("REDIRECT_ANONYMOUS");
}
