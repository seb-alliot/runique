//! Tests — utils/config/runique_log
//! Couvre : RuniqueLog builder (arbre par module), dev(), subscriber_level

use runique::utils::RuniqueLog;
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
    log.init_subscriber();
}
