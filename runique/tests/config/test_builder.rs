//! Tests — app/builder.rs + app/error_build.rs + app/staging/*.rs
//!
//! Couvre :
//! - BuildError / BuildErrorKind / CheckError / CheckReport
//! - StaticStaging
//! - CoreStaging (sans et avec DB)
//! - MiddlewareStaging builder methods
//! - MiddlewareConfig (development / production)
//! - AdminStaging (new, enable/disable, validate, is_ready, hot_reload, routes…)
//! - RuniqueAppBuilder (construction, méthodes chainables, build())

use async_trait::async_trait;
use axum::{Router, routing::get};
use runique::admin::AdminConfig;
use runique::app::staging::{AdminStaging, CoreStaging, MiddlewareStaging, StaticStaging};
use runique::app::{BuildError, BuildErrorKind, CheckError, CheckReport, RuniqueAppBuilder};
use runique::config::app::RuniqueConfig;
use runique::middleware::MiddlewareConfig;
use runique::middleware::auth::{AdminAuth, AdminLoginResult};
use sea_orm::DatabaseConnection;
use tower_sessions::cookie::time::Duration;

// ─── Mock AdminAuth pour les tests ────────────────────────────────────────────

struct MockAdminAuth;

#[async_trait]
impl AdminAuth for MockAdminAuth {
    async fn authenticate(
        &self,
        _username: &str,
        _password: &str,
        _db: &DatabaseConnection,
    ) -> Option<AdminLoginResult> {
        None
    }
}

// ════════════════════════════════════════════════════════════════
// CheckError
// ════════════════════════════════════════════════════════════════

#[test]
fn test_check_error_new_champs() {
    let err = CheckError::new("Database", "Connexion manquante");
    assert_eq!(err.component, "Database");
    assert_eq!(err.message, "Connexion manquante");
    assert!(err.suggestion.is_none());
}

#[test]
fn test_check_error_with_suggestion() {
    let err = CheckError::new("Database", "Manquante").with_suggestion("Ajoutez .with_database()");
    assert_eq!(err.suggestion, Some("Ajoutez .with_database()".to_string()));
}

#[test]
fn test_check_error_display_sans_suggestion() {
    let err = CheckError::new("Session", "Store manquant");
    let s = format!("{}", err);
    assert!(s.contains("Session"));
    assert!(s.contains("Store manquant"));
}

#[test]
fn test_check_error_display_avec_suggestion() {
    let err = CheckError::new("DB", "Absent").with_suggestion("Utilisez .with_database()");
    let s = format!("{}", err);
    assert!(s.contains("Suggestion"));
    assert!(s.contains("with_database"));
}

#[test]
fn test_check_error_debug() {
    let err = CheckError::new("X", "Y");
    assert!(format!("{:?}", err).contains("CheckError"));
}

// ════════════════════════════════════════════════════════════════
// CheckReport
// ════════════════════════════════════════════════════════════════

#[test]
fn test_check_report_new_vide() {
    let report = CheckReport::new();
    assert!(!report.has_errors());
    assert_eq!(report.count(), 0);
    assert!(report.errors.is_empty());
}

#[test]
fn test_check_report_default_vide() {
    let report = CheckReport::default();
    assert!(!report.has_errors());
}

#[test]
fn test_check_report_add_une_erreur() {
    let mut report = CheckReport::new();
    report.add(CheckError::new("DB", "Manquante"));
    assert!(report.has_errors());
    assert_eq!(report.count(), 1);
}

#[test]
fn test_check_report_add_plusieurs_erreurs() {
    let mut report = CheckReport::new();
    report.add(CheckError::new("DB", "Manquante"));
    report.add(CheckError::new("Templates", "Absents"));
    assert_eq!(report.count(), 2);
}

#[test]
fn test_check_report_debug() {
    let report = CheckReport::new();
    assert!(format!("{:?}", report).contains("CheckReport"));
}

// ════════════════════════════════════════════════════════════════
// BuildError
// ════════════════════════════════════════════════════════════════

#[test]
fn test_build_error_validation() {
    let err = BuildError::validation("Config invalide");
    assert!(matches!(err.kind, BuildErrorKind::ValidationFailed(_)));
    assert!(err.context.is_none());
}

#[test]
fn test_build_error_validation_display() {
    let err = BuildError::validation("Config invalide");
    let s = format!("{}", err);
    assert!(s.contains("Build validation failed"));
    assert!(s.contains("Config invalide"));
}

#[test]
fn test_build_error_template() {
    let err = BuildError::template("Template introuvable");
    assert!(matches!(err.kind, BuildErrorKind::TemplateLoadFailed(_)));
}

#[test]
fn test_build_error_template_display() {
    let err = BuildError::template("Erreur tera");
    let s = format!("{}", err);
    assert!(s.contains("Template loading failed"));
    assert!(s.contains("Erreur tera"));
}

#[test]
fn test_build_error_database_missing() {
    let err = BuildError::database_missing();
    assert!(matches!(err.kind, BuildErrorKind::DatabaseMissing));
}

#[test]
fn test_build_error_database_missing_display() {
    let err = BuildError::database_missing();
    let s = format!("{}", err);
    assert!(s.contains("Database") || s.contains("database"));
}

#[test]
fn test_build_error_check() {
    let mut report = CheckReport::new();
    report.add(CheckError::new("DB", "Absente"));
    let err = BuildError::check(report);
    assert!(matches!(err.kind, BuildErrorKind::CheckFailed(_)));
}

#[test]
fn test_build_error_check_display_avec_erreurs() {
    let mut report = CheckReport::new();
    report.add(
        CheckError::new("Database", "Connexion manquante")
            .with_suggestion("Ajoutez .with_database()"),
    );
    let err = BuildError::check(report);
    let s = format!("{}", err);
    assert!(s.contains("check error") || s.contains("1"));
    assert!(s.contains("Database"));
}

#[test]
fn test_build_error_with_context() {
    let err = BuildError::validation("Erreur").with_context("Contexte additionnel");
    assert_eq!(err.context, Some("Contexte additionnel".to_string()));
}

#[test]
fn test_build_error_with_context_display() {
    let err = BuildError::validation("Test").with_context("Mon contexte");
    let s = format!("{}", err);
    assert!(s.contains("Context: Mon contexte"));
}

#[test]
fn test_build_error_from_tera_error() {
    let tera_err = tera::Error::msg("Template manquant");
    let build_err: BuildError = tera_err.into();
    assert!(matches!(
        build_err.kind,
        BuildErrorKind::TemplateLoadFailed(_)
    ));
}

#[test]
fn test_build_error_debug() {
    let err = BuildError::validation("X");
    assert!(format!("{:?}", err).contains("BuildError"));
}

#[test]
fn test_build_error_est_error_trait() {
    let err = BuildError::validation("test");
    let _: &dyn std::error::Error = &err;
}

// ════════════════════════════════════════════════════════════════
// StaticStaging
// ════════════════════════════════════════════════════════════════

#[test]
fn test_static_staging_new_enabled_par_defaut() {
    assert!(StaticStaging::new().is_enabled());
}

#[test]
fn test_static_staging_default_enabled() {
    assert!(StaticStaging::default().is_enabled());
}

#[test]
fn test_static_staging_disable() {
    assert!(!StaticStaging::new().disable().is_enabled());
}

#[test]
fn test_static_staging_enable_apres_disable() {
    assert!(StaticStaging::new().disable().enable().is_enabled());
}

#[test]
fn test_static_staging_enabled_false() {
    assert!(!StaticStaging::new().enabled(false).is_enabled());
}

#[test]
fn test_static_staging_enabled_true() {
    assert!(StaticStaging::new().disable().enabled(true).is_enabled());
}

#[test]
fn test_static_staging_validate_ok() {
    let s = StaticStaging::new();
    assert!(s.validate().is_ok());
}

#[test]
fn test_static_staging_validate_disabled_ok() {
    let s = StaticStaging::new().disable();
    assert!(s.validate().is_ok());
}

#[test]
fn test_static_staging_is_ready_toujours_true() {
    assert!(StaticStaging::new().is_ready());
    assert!(StaticStaging::new().disable().is_ready());
}

// ════════════════════════════════════════════════════════════════
// CoreStaging
// ════════════════════════════════════════════════════════════════

#[test]
fn test_core_staging_new_sans_db() {
    let core = CoreStaging::new();
    assert!(!core.is_ready()); // ORM activé → false sans DB
}

#[test]
fn test_core_staging_default() {
    let core = CoreStaging::default();
    assert!(!core.is_ready());
}

#[test]
fn test_core_staging_validate_sans_db_retourne_err() {
    let core = CoreStaging::new();
    let result = core.validate();
    assert!(result.is_err());
    if let Err(build_err) = result {
        if let BuildErrorKind::CheckFailed(report) = &build_err.kind {
            assert!(report.has_errors());
            assert!(
                report
                    .errors
                    .iter()
                    .any(|e| e.component.contains("Database"))
            );
        } else {
            panic!("Attendu CheckFailed, obtenu {:?}", build_err.kind);
        }
    }
}

#[tokio::test]
async fn test_core_staging_with_database_is_ready() {
    use sea_orm::Database;
    let db = Database::connect("sqlite::memory:").await.unwrap();
    let core = CoreStaging::new().with_database(db);
    assert!(core.is_ready());
    assert!(core.validate().is_ok());
}

// ════════════════════════════════════════════════════════════════
// MiddlewareConfig (development / production profiles)
// ════════════════════════════════════════════════════════════════

#[test]
fn test_middleware_config_development() {
    let config = MiddlewareConfig::development();
    assert!(!config.enable_csp);
    assert!(!config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(!config.enable_cache);
}

#[test]
fn test_middleware_config_production() {
    let config = MiddlewareConfig::production();
    assert!(config.enable_csp);
    assert!(!config.enable_header_security); // false par défaut — activé via builder
    assert!(config.enable_host_validation);
    assert!(config.enable_debug_errors);
    assert!(config.enable_cache);
}

// ════════════════════════════════════════════════════════════════
// MiddlewareStaging — API publique (champs internes = pub(crate))
// ════════════════════════════════════════════════════════════════

#[test]
fn test_middleware_staging_new_debug_profil_dev() {
    let ms = MiddlewareStaging::new(true);
    assert!(!ms.features().enable_csp);
    assert!(!ms.features().enable_host_validation);
    assert!(ms.features().enable_debug_errors);
    assert!(!ms.features().enable_cache);
}

#[test]
fn test_middleware_staging_new_prod_profil_prod() {
    let ms = MiddlewareStaging::new(false);
    assert!(ms.features().enable_csp);
    assert!(ms.features().enable_host_validation);
    assert!(ms.features().enable_debug_errors);
    assert!(ms.features().enable_cache);
}

#[test]
fn test_middleware_staging_from_config_debug() {
    let mut config = RuniqueConfig {
        debug: true,
        ..Default::default()
    };
    config.debug = true;
    let ms = MiddlewareStaging::from_config(&config);
    assert!(!ms.features().enable_csp);
}

#[test]
fn test_middleware_staging_from_config_prod() {
    let mut config = RuniqueConfig {
        debug: true,
        ..Default::default()
    };
    config.debug = false;
    let ms = MiddlewareStaging::from_config(&config);
    // CSP configure uniquement via le builder — toujours false depuis from_config
    assert!(!ms.features().enable_csp);
    assert!(!ms.features().enable_header_security);
}

#[test]
fn test_middleware_staging_with_csp_active() {
    let ms = MiddlewareStaging::new(true).with_csp(|c| c);
    assert!(ms.features().enable_csp);
    assert!(!ms.features().enable_header_security);
}

#[test]
fn test_middleware_staging_with_csp_avec_header_security() {
    let ms = MiddlewareStaging::new(true).with_csp(|c| c.with_header_security(true));
    assert!(ms.features().enable_csp);
    assert!(ms.features().enable_header_security);
}

#[test]
fn test_middleware_staging_without_csp_desactive() {
    // Ne pas appeler with_csp = CSP desactive
    let ms = MiddlewareStaging::new(true);
    assert!(!ms.features().enable_csp);
    assert!(!ms.features().enable_header_security);
}

#[test]
fn test_middleware_staging_with_host_validation_true() {
    let ms = MiddlewareStaging::new(true).with_host_validation(true);
    assert!(ms.features().enable_host_validation);
}

#[test]
fn test_middleware_staging_with_host_validation_false() {
    let ms = MiddlewareStaging::new(true).with_host_validation(false);
    assert!(!ms.features().enable_host_validation);
}

#[test]
fn test_middleware_staging_with_debug_errors_false() {
    let ms = MiddlewareStaging::new(false).with_debug_errors(false);
    assert!(!ms.features().enable_debug_errors);
}

#[test]
fn test_middleware_staging_with_debug_errors_true() {
    let ms = MiddlewareStaging::new(false).with_debug_errors(true);
    assert!(ms.features().enable_debug_errors);
}

#[test]
fn test_middleware_staging_with_cache_true() {
    let ms = MiddlewareStaging::new(true).with_cache(true);
    assert!(ms.features().enable_cache);
}

#[test]
fn test_middleware_staging_with_cache_false() {
    let ms = MiddlewareStaging::new(false).with_cache(false);
    assert!(!ms.features().enable_cache);
}

#[test]
fn test_middleware_staging_with_session_duration() {
    let ms = MiddlewareStaging::new(true).with_session_duration(Duration::hours(2));
    assert_eq!(ms.session_duration(), Duration::hours(2));
}

#[test]
fn test_middleware_staging_session_duration_defaut_24h() {
    let ms = MiddlewareStaging::new(true);
    assert_eq!(ms.session_duration(), Duration::seconds(86400));
}

#[test]
fn test_middleware_staging_add_custom_incremente_count() {
    let ms = MiddlewareStaging::new(true)
        .add_custom(|r| r)
        .add_custom(|r| r);
    assert_eq!(ms.custom_count(), 2);
}

#[test]
fn test_middleware_staging_validate_ok() {
    let ms = MiddlewareStaging::new(true);
    assert!(ms.validate().is_ok());
}

#[test]
fn test_middleware_staging_validate_prod_ok() {
    let ms = MiddlewareStaging::new(false);
    assert!(ms.validate().is_ok());
}

#[test]
fn test_middleware_staging_is_ready_toujours_true() {
    assert!(MiddlewareStaging::new(true).is_ready());
    assert!(MiddlewareStaging::new(false).is_ready());
}

// ════════════════════════════════════════════════════════════════
// RuniqueAppBuilder — construction et méthodes chainables (sync)
// ════════════════════════════════════════════════════════════════

fn make_config() -> RuniqueConfig {
    RuniqueConfig::default()
}

#[test]
fn test_builder_new_ne_panique_pas() {
    let _b = RuniqueAppBuilder::new(make_config());
}

#[test]
fn test_builder_routes_chainable() {
    let router = Router::new().route("/", get(|| async { "ok" }));
    let _b = RuniqueAppBuilder::new(make_config()).routes(router);
}

#[test]
fn test_builder_no_statics_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).no_statics();
}

#[test]
fn test_builder_statics_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).statics();
}

#[test]
fn test_builder_with_session_duration_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).with_session_duration(Duration::hours(1));
}

#[test]
fn test_builder_with_error_handler_true_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).with_error_handler(true);
}

#[test]
fn test_builder_with_error_handler_false_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).with_error_handler(false);
}

#[test]
fn test_builder_middleware_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).middleware(|m| m.with_csp(|c| c));
}

#[test]
fn test_builder_static_files_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).static_files(|s| s.disable());
}

#[test]
fn test_builder_core_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).core(|c| c);
}

#[test]
fn test_builder_with_admin_chainable() {
    let _b = RuniqueAppBuilder::new(make_config()).with_admin(|a| a);
}

#[test]
fn test_builder_chaine_complete_sync() {
    let router = Router::new().route("/", get(|| async { "ok" }));
    let _b = RuniqueAppBuilder::new(make_config())
        .routes(router)
        .no_statics()
        .with_session_duration(Duration::hours(8))
        .with_error_handler(false)
        .middleware(|m| {
            m.with_cache(false)
                .with_csp(|c| c.with_header_security(true))
        })
        .static_files(|s| s.disable())
        .core(|c| c)
        .with_admin(|a| a);
}

// ════════════════════════════════════════════════════════════════
// RuniqueAppBuilder::build() — tests async
// ════════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_build_sans_db_retourne_check_failed() {
    let config = make_config();
    let result = RuniqueAppBuilder::new(config)
        .routes(Router::new())
        .no_statics()
        .build()
        .await;

    assert!(result.is_err(), "Sans DB, build() doit échouer");
    let err = match result {
        Ok(_) => panic!("Attendu Err, obtenu Ok"),
        Err(e) => e,
    };
    assert!(
        matches!(err.kind, BuildErrorKind::CheckFailed(_)),
        "Attendu CheckFailed, obtenu {:?}",
        err.kind
    );

    if let BuildErrorKind::CheckFailed(report) = &err.kind {
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.component.contains("Database")),
            "Le rapport doit mentionner la Database manquante"
        );
    }
}

#[tokio::test]
async fn test_build_avec_database_retourne_ok_ou_template_err() {
    use sea_orm::Database;

    let db = Database::connect("sqlite::memory:").await.unwrap();

    let router = Router::new().route("/", get(|| async { "ok" }));
    let result = RuniqueAppBuilder::new(make_config())
        .routes(router)
        .no_statics()
        .core(|c| c.with_database(db))
        .build()
        .await;

    // Selon l'environnement de test, le build peut réussir (templates OK)
    // ou échouer sur le chargement des templates Tera (acceptable).
    // Ce qui est inacceptable : CheckFailed (DB manquante) ou panique.
    match result {
        Ok(_app) => {
            // Build complet réussi
        }
        Err(ref e) => {
            assert!(
                !matches!(e.kind, BuildErrorKind::CheckFailed(_)),
                "Ne doit pas échouer sur CheckFailed avec une DB valide"
            );
            // TemplateLoadFailed ou autre erreur d'environnement = acceptable
        }
    }
}

#[tokio::test]
async fn test_build_check_failed_display_utile() {
    let config = make_config();
    let result = RuniqueAppBuilder::new(config).no_statics().build().await;

    assert!(result.is_err());
    let err = match result {
        Ok(_) => panic!("Attendu Err"),
        Err(e) => e,
    };
    let msg = format!("{}", err);
    assert!(!msg.is_empty());
}

/// Build avec statics activés → couvre attach_static_files
#[tokio::test]
async fn test_build_avec_statics_couvre_attach_static_files() {
    use sea_orm::Database;
    let db = Database::connect("sqlite::memory:").await.unwrap();

    // RuniqueConfig::default() génère des URLs vides → nest_service("") panique.
    // On configure des valeurs valides pour éviter le panic Axum.
    let mut config = make_config();
    config.static_files.static_url = "/static".to_string();
    config.static_files.media_url = "/media".to_string();
    config.static_files.staticfiles_dirs = ".".to_string();
    config.static_files.media_root = ".".to_string();
    config.static_files.static_runique_url = String::new(); // désactive la 3e route

    let result = RuniqueAppBuilder::new(config)
        .routes(Router::new())
        .statics() // statics enabled → exerce attach_static_files()
        .core(|c| c.with_database(db))
        .build()
        .await;

    match result {
        Ok(_) => {} // succès
        Err(ref e) => {
            assert!(
                !matches!(e.kind, BuildErrorKind::CheckFailed(_)),
                "Ne doit pas échouer sur CheckFailed avec DB valide"
            );
        }
    }
}

/// Build avec CSP + header_security activés via builder → couvre security_headers_middleware
#[tokio::test]
async fn test_build_profil_production_couvre_csp_host_validation() {
    use sea_orm::Database;
    let db = Database::connect("sqlite::memory:").await.unwrap();

    let mut config = make_config();
    config.debug = false;

    let result = RuniqueAppBuilder::new(config)
        .routes(Router::new())
        .no_statics()
        .core(|c| c.with_database(db))
        .middleware(|m| m.with_csp(|c| c.with_header_security(true).with_nonce(true)))
        .build()
        .await;

    match result {
        Ok(_) => {}
        Err(ref e) => {
            assert!(
                !matches!(e.kind, BuildErrorKind::CheckFailed(_)),
                "Ne doit pas echouer sur CheckFailed avec DB valide"
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════
// AdminStaging
// ════════════════════════════════════════════════════════════════

#[test]
fn test_admin_staging_new_disabled_par_defaut() {
    let a = AdminStaging::new();
    assert!(!a.enabled);
}

#[test]
fn test_admin_staging_default_disabled() {
    let a = AdminStaging::default();
    assert!(!a.enabled);
}

#[test]
fn test_admin_staging_enable() {
    let a = AdminStaging::new().enable();
    assert!(a.enabled);
}

#[test]
fn test_admin_staging_disable_apres_enable() {
    let a = AdminStaging::new().enable().disable();
    assert!(!a.enabled);
}

#[test]
fn test_admin_staging_hot_reload() {
    let a = AdminStaging::new().hot_reload(true);
    assert!(a.config.hot_reload);
}

#[test]
fn test_admin_staging_site_title() {
    let a = AdminStaging::new().site_title("Mon Admin");
    assert_eq!(a.config.site_title, "Mon Admin");
}

#[test]
fn test_admin_staging_routes_stocke_router() {
    let router = Router::new().route("/admin/test", get(|| async { "ok" }));
    let a = AdminStaging::new().routes(router);
    assert!(a.route_admin.is_some());
}

#[test]
fn test_admin_staging_with_proto_state_none_par_defaut() {
    let a = AdminStaging::new();
    assert!(a.state.is_none());
}

#[test]
fn test_admin_staging_is_ready_disabled() {
    assert!(AdminStaging::new().is_ready()); // disabled → toujours prêt
}

#[test]
fn test_admin_staging_is_ready_enabled_avec_prefix() {
    let a = AdminStaging::new().enable();
    // prefix par défaut = "/admin" → non vide → is_ready = true
    assert!(a.is_ready());
}

#[test]
fn test_admin_staging_validate_disabled_ok() {
    let a = AdminStaging::new();
    assert!(a.validate().is_ok());
}

#[test]
fn test_admin_staging_validate_enabled_sans_auth_retourne_err() {
    let a = AdminStaging::new().enable();
    let result = a.validate();
    assert!(result.is_err());
    if let Err(e) = result {
        if let BuildErrorKind::CheckFailed(report) = &e.kind {
            assert!(
                report
                    .errors
                    .iter()
                    .any(|e| e.component.contains("AdminPanel"))
            );
        } else {
            panic!("Attendu CheckFailed, obtenu {:?}", e.kind);
        }
    }
}

#[test]
fn test_admin_staging_auth_avec_mock() {
    let a = AdminStaging::new().auth(MockAdminAuth);
    assert!(a.config.auth.is_some());
}

#[test]
fn test_admin_staging_validate_enabled_avec_auth_ok() {
    let a = AdminStaging::new().enable().auth(MockAdminAuth);
    assert!(a.validate().is_ok());
}

#[test]
fn test_admin_config_new_prefix_defaut() {
    let c = AdminConfig::new();
    assert_eq!(c.prefix, "/admin");
}

#[test]
fn test_admin_config_hot_reload() {
    let c = AdminConfig::new().hot_reload(true);
    assert!(c.hot_reload);
}

#[test]
fn test_admin_config_site_title() {
    let c = AdminConfig::new().site_title("Runique Admin");
    assert_eq!(c.site_title, "Runique Admin");
}

#[test]
fn test_admin_config_disable() {
    let c = AdminConfig::new().disable();
    assert!(!c.enabled);
}

#[test]
fn test_admin_config_auth() {
    let c = AdminConfig::new().auth(MockAdminAuth);
    assert!(c.auth.is_some());
}

#[test]
fn test_admin_config_debug() {
    let c = AdminConfig::new();
    assert!(format!("{:?}", c).contains("AdminConfig"));
}

#[test]
fn test_admin_config_clone() {
    let c = AdminConfig::new().site_title("Clone Test");
    let c2 = c.clone();
    assert_eq!(c2.site_title, "Clone Test");
}
