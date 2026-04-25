//! Tests — utils/config/runique_log
//! Couvre : RuniqueLog builder, dev(), champs optionnels, subscriber_level

use runique::utils::RuniqueLog;
use tracing::Level;

#[test]
fn test_default_all_none() {
    let log = RuniqueLog::new();
    assert!(log.csrf.is_none());
    assert!(log.session.is_none());
    assert!(log.db.is_none());
    assert!(log.exclusive_login.is_none());
    assert!(log.filter_fn.is_none());
    assert!(log.roles.is_none());
    assert!(log.password_init.is_none());
    assert!(log.host_validation.is_none());
    assert!(log.acme.is_none());
}

#[test]
fn test_csrf_builder() {
    let log = RuniqueLog::new().csrf(Level::WARN);
    assert_eq!(log.csrf, Some(Level::WARN));
}

#[test]
fn test_session_builder() {
    let log = RuniqueLog::new().session(Level::INFO);
    assert_eq!(log.session, Some(Level::INFO));
}

#[test]
fn test_db_builder() {
    let log = RuniqueLog::new().db(Level::DEBUG);
    assert_eq!(log.db, Some(Level::DEBUG));
}

#[test]
fn test_exclusive_login_builder() {
    let log = RuniqueLog::new().exclusive_login(Level::ERROR);
    assert_eq!(log.exclusive_login, Some(Level::ERROR));
}

#[test]
fn test_filter_fn_builder() {
    let log = RuniqueLog::new().filter_fn(Level::WARN);
    assert_eq!(log.filter_fn, Some(Level::WARN));
}

#[test]
fn test_roles_builder() {
    let log = RuniqueLog::new().roles(Level::INFO);
    assert_eq!(log.roles, Some(Level::INFO));
}

#[test]
fn test_password_init_builder() {
    let log = RuniqueLog::new().password_init(Level::WARN);
    assert_eq!(log.password_init, Some(Level::WARN));
}

#[test]
fn test_host_validation_builder() {
    let log = RuniqueLog::new().host_validation(Level::DEBUG);
    assert_eq!(log.host_validation, Some(Level::DEBUG));
}

#[test]
fn test_acme_builder() {
    let log = RuniqueLog::new().acme(Level::INFO);
    assert_eq!(log.acme, Some(Level::INFO));
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
        .csrf(Level::WARN)
        .session(Level::INFO)
        .db(Level::DEBUG)
        .acme(Level::ERROR);
    assert_eq!(log.csrf, Some(Level::WARN));
    assert_eq!(log.session, Some(Level::INFO));
    assert_eq!(log.db, Some(Level::DEBUG));
    assert_eq!(log.acme, Some(Level::ERROR));
}

#[test]
fn test_dev_does_not_panic() {
    // dev() is a no-op when DEBUG env is not set (LazyLock evaluated once at startup).
    let log = RuniqueLog::new().dev();
    // Either all None (no debug) or all Some(DEBUG) — just ensure it doesn't panic.
    drop(log);
}

#[test]
fn test_init_subscriber_does_not_panic() {
    let log = RuniqueLog::new().subscriber_level("error");
    log.init_subscriber();
}
