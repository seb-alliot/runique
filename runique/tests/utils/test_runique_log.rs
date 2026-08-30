//! Tests — utils/config/runique_log
//! Couvre : RuniqueLog builder (arbre par module), dev(), subscriber_level

use runique::utils::{MigrationTracing, RuniqueLog, TemplatesTracing};
use tracing::Level;

#[test]
fn test_default_all_none() {
    let log = RuniqueLog::new();
    assert!(log.middleware.is_none());
    assert!(log.session.is_none());
    assert!(log.db.is_none());
    assert!(log.admin.is_none());
    assert!(log.auth.is_none());
    assert!(log.forms.is_none());
}

#[test]
fn test_csrf_builder() {
    let log = RuniqueLog::new().middleware(|m| m.csrf(Level::WARN));
    assert_eq!(
        log.middleware.as_ref().and_then(|m| m.csrf),
        Some(Level::WARN)
    );
}

#[test]
fn test_session_builder() {
    let log = RuniqueLog::new().session(|s| s.store(Level::INFO));
    assert_eq!(
        log.session.as_ref().and_then(|s| s.store),
        Some(Level::INFO)
    );
}

#[test]
fn test_db_builder() {
    let log = RuniqueLog::new().db(|d| d.connect(Level::DEBUG));
    assert_eq!(log.db.as_ref().and_then(|d| d.connect), Some(Level::DEBUG));
}

#[test]
fn test_exclusive_login_builder() {
    let log = RuniqueLog::new().session(|s| s.exclusive_login(Level::ERROR));
    assert_eq!(
        log.session.as_ref().and_then(|s| s.exclusive_login),
        Some(Level::ERROR)
    );
}

#[test]
fn test_filter_fn_builder() {
    let log = RuniqueLog::new().admin(|a| a.filter_fn(Level::WARN));
    assert_eq!(
        log.admin.as_ref().and_then(|a| a.filter_fn),
        Some(Level::WARN)
    );
}

#[test]
fn test_roles_builder() {
    let log = RuniqueLog::new().admin(|a| a.roles(Level::INFO));
    assert_eq!(log.admin.as_ref().and_then(|a| a.roles), Some(Level::INFO));
}

#[test]
fn test_password_init_builder() {
    let log = RuniqueLog::new().auth(|a| a.password_init(Level::WARN));
    assert_eq!(
        log.auth.as_ref().and_then(|a| a.password_init),
        Some(Level::WARN)
    );
}

#[test]
fn test_host_validation_builder() {
    let log = RuniqueLog::new().middleware(|m| m.host_validation(Level::DEBUG));
    assert_eq!(
        log.middleware.as_ref().and_then(|m| m.host_validation),
        Some(Level::DEBUG)
    );
}

#[test]
fn test_https_builder() {
    let log = RuniqueLog::new().middleware(|m| m.https(Level::INFO));
    assert_eq!(
        log.middleware.as_ref().and_then(|m| m.https),
        Some(Level::INFO)
    );
}

#[test]
fn test_subscriber_level_builder() {
    let log = RuniqueLog::new().subscriber_level("info");
    // subscriber_level is private — just check it doesn't panic and compiles
    drop(log);
}

#[test]
fn test_chained_builders() {
    let log = RuniqueLog::new()
        .middleware(|m| m.csrf(Level::WARN).https(Level::ERROR))
        .session(|s| s.store(Level::INFO))
        .db(|d| d.connect(Level::DEBUG));
    assert_eq!(
        log.middleware.as_ref().and_then(|m| m.csrf),
        Some(Level::WARN)
    );
    assert_eq!(
        log.middleware.as_ref().and_then(|m| m.https),
        Some(Level::ERROR)
    );
    assert_eq!(
        log.session.as_ref().and_then(|s| s.store),
        Some(Level::INFO)
    );
    assert_eq!(log.db.as_ref().and_then(|d| d.connect), Some(Level::DEBUG));
}

#[test]
fn test_dev_does_not_panic() {
    // dev() is a no-op when DEBUG env is not set.
    let log = RuniqueLog::new().dev();
    drop(log);
}

#[test]
fn test_init_subscriber_does_not_panic() {
    let log = RuniqueLog::new().subscriber_level("error");
    let _guards = log.init_subscriber();
}

// ─── MigrationTracing ──────────────────────────────────────────────────────────

#[test]
fn test_migration_tracing_default_all_none() {
    let mt = MigrationTracing::new();
    assert!(mt.plan.is_none());
    assert!(mt.apply.is_none());
    assert!(mt.rollback.is_none());
}

#[test]
fn test_migration_tracing_builder_chain() {
    let mt = MigrationTracing::new()
        .plan(Level::INFO)
        .apply(Level::WARN)
        .rollback(Level::ERROR);
    assert_eq!(mt.plan, Some(Level::INFO));
    assert_eq!(mt.apply, Some(Level::WARN));
    assert_eq!(mt.rollback, Some(Level::ERROR));
}

#[test]
fn test_migration_tracing_dev_sets_all_debug() {
    let mt = MigrationTracing::new().dev();
    assert_eq!(mt.plan, Some(Level::DEBUG));
    assert_eq!(mt.apply, Some(Level::DEBUG));
    assert_eq!(mt.rollback, Some(Level::DEBUG));
}

// ─── TemplatesTracing ──────────────────────────────────────────────────────────

#[test]
fn test_templates_tracing_default_all_none() {
    let tt = TemplatesTracing::new();
    assert!(tt.load.is_none());
    assert!(tt.render.is_none());
}

#[test]
fn test_templates_tracing_builder_chain() {
    let tt = TemplatesTracing::new().load(Level::INFO).render(Level::WARN);
    assert_eq!(tt.load, Some(Level::INFO));
    assert_eq!(tt.render, Some(Level::WARN));
}

#[test]
fn test_templates_tracing_dev_sets_all_debug() {
    let tt = TemplatesTracing::new().dev();
    assert_eq!(tt.load, Some(Level::DEBUG));
    assert_eq!(tt.render, Some(Level::DEBUG));
}
