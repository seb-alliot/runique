//! Tests — utils/runique_log
//! Couvre : log_init, get_log, reset_log_for_test, activation par domaine

use runique::utils::runique_log::{RuniqueLog, get_log, log_init, reset_log_for_test};
use serial_test::serial;
use tracing::Level;

// ─── log_init / get_log ───────────────────────────────────────────────────────

#[test]
#[serial]
fn log_init_active_la_config() {
    reset_log_for_test();
    log_init(RuniqueLog::new().db(Level::DEBUG));
    assert_eq!(get_log().db, Some(Level::DEBUG));
    reset_log_for_test();
}

#[test]
#[serial]
fn log_init_idempotent() {
    reset_log_for_test();
    log_init(RuniqueLog::new().db(Level::DEBUG));
    log_init(RuniqueLog::new().db(Level::ERROR)); // ignoré
    assert_eq!(get_log().db, Some(Level::DEBUG));
    reset_log_for_test();
}

#[test]
#[serial]
fn get_log_sans_init_retourne_default() {
    reset_log_for_test();
    let log = get_log();
    assert!(log.db.is_none());
    assert!(log.csrf.is_none());
    assert!(log.forms.is_none());
}

// ─── FormTracing ──────────────────────────────────────────────────────────────

#[test]
#[serial]
fn form_tracing_champs_independants() {
    reset_log_for_test();
    log_init(RuniqueLog::new().forms(|f| f.validate(Level::DEBUG).render(Level::INFO)));
    let log = get_log();
    let forms = log.forms.as_ref().unwrap();
    assert_eq!(forms.validate, Some(Level::DEBUG));
    assert_eq!(forms.render, Some(Level::INFO));
    assert!(forms.field.is_none());
    assert!(forms.finalize.is_none());
    reset_log_for_test();
}

#[test]
#[serial]
fn form_tracing_dev_active_tout() {
    reset_log_for_test();
    log_init(RuniqueLog::new().forms(|f| f.dev()));
    let forms = get_log().forms.clone().unwrap();
    assert_eq!(forms.field, Some(Level::DEBUG));
    assert_eq!(forms.set_value, Some(Level::DEBUG));
    assert_eq!(forms.validate, Some(Level::DEBUG));
    assert_eq!(forms.render, Some(Level::DEBUG));
    assert_eq!(forms.finalize, Some(Level::DEBUG));
    reset_log_for_test();
}

// ─── AuthTracing ──────────────────────────────────────────────────────────────

#[test]
#[serial]
fn auth_tracing_login_et_reset() {
    reset_log_for_test();
    log_init(RuniqueLog::new().auth(|a| a.login(Level::INFO).reset(Level::WARN)));
    let auth = get_log().auth.clone().unwrap();
    assert_eq!(auth.login, Some(Level::INFO));
    assert_eq!(auth.reset, Some(Level::WARN));
    reset_log_for_test();
}

// ─── BuilderTracing ───────────────────────────────────────────────────────────

#[test]
#[serial]
fn builder_tracing_dev_active_tout() {
    reset_log_for_test();
    log_init(RuniqueLog::new().builder(|b| b.dev()));
    let b = get_log().builder.clone().unwrap();
    assert_eq!(b.templates, Some(Level::DEBUG));
    assert_eq!(b.registry, Some(Level::DEBUG));
    assert_eq!(b.middleware, Some(Level::DEBUG));
    assert_eq!(b.statics, Some(Level::DEBUG));
    assert_eq!(b.routes, Some(Level::DEBUG));
    reset_log_for_test();
}

// ─── rate_limit ───────────────────────────────────────────────────────────────

#[test]
#[serial]
fn rate_limit_desactive_par_defaut() {
    reset_log_for_test();
    assert!(get_log().rate_limit.is_none());
}

#[test]
#[serial]
fn rate_limit_active() {
    reset_log_for_test();
    log_init(RuniqueLog::new().rate_limit(Level::WARN));
    assert_eq!(get_log().rate_limit, Some(Level::WARN));
    reset_log_for_test();
}

// ─── MailerTracing ────────────────────────────────────────────────────────────

#[test]
#[serial]
fn mailer_tracing_send() {
    reset_log_for_test();
    log_init(RuniqueLog::new().mailer(|m| m.send(Level::INFO)));
    assert_eq!(get_log().mailer.as_ref().unwrap().send, Some(Level::INFO));
    reset_log_for_test();
}

// ─── dev() global ────────────────────────────────────────────────────────────

#[test]
#[serial]
fn dev_active_tous_les_domaines() {
    reset_log_for_test();
    // RuniqueLog::dev() dépend d'un LazyLock initialisé une seule fois —
    // on construit la config manuellement pour ne pas dépendre de DEBUG env
    log_init(
        RuniqueLog::new()
            .forms(|f| f.dev())
            .auth(|a| a.dev())
            .builder(|b| b.dev())
            .rate_limit(Level::DEBUG),
    );
    let log = get_log();
    assert!(log.forms.as_ref().and_then(|f| f.validate).is_some());
    assert!(log.auth.as_ref().and_then(|a| a.login).is_some());
    assert!(log.builder.as_ref().and_then(|b| b.templates).is_some());
    assert!(log.rate_limit.is_some());
    reset_log_for_test();
}
