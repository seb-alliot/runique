//! A password reset link is never built from the request's `Host` in
//! production: the app refuses to boot without `SITE_URL` when it can send
//! one — through the public reset pages or the admin.

use axum::Router;
use runique::app::{BuildErrorKind, RuniqueApp, RuniqueAppBuilder};
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;

async fn builder(debug: bool) -> RuniqueAppBuilder {
    let mut config = RuniqueConfig::from_env();
    config.debug = debug;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    config.server.site_url = None;
    RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new())
        .static_files(|s| s.enabled(false))
        .middleware(|m| m.with_session_store(tower_sessions::MemoryStore::default()))
}

/// The components the boot check refused, or none when the app built.
async fn refused(b: RuniqueAppBuilder) -> Vec<String> {
    match b.build().await {
        Ok(_) => Vec::new(),
        Err(e) => match e.kind {
            BuildErrorKind::CheckFailed(report) => {
                report.errors.into_iter().map(|e| e.component).collect()
            }
            other => panic!("unexpected build error: {other:?}"),
        },
    }
}

#[tokio::test]
#[serial]
async fn production_refuses_a_password_reset_or_an_admin_without_site_url() {
    let refused_without = |components: Vec<String>| components.contains(&"SiteUrl".to_string());
    assert!(refused_without(
        refused(builder(false).await.with_password_reset(|pr| pr)).await
    ));
    assert!(refused_without(
        refused(builder(false).await.with_admin(|a| a)).await
    ));

    for b in [
        builder(false).await.with_password_reset(|pr| pr),
        builder(false).await.with_admin(|a| a),
    ] {
        let components = refused(b.site_url("https://mysite.com")).await;
        assert!(!refused_without(components.clone()), "{components:?}");
    }
}

// Nothing that sends a link: no SITE_URL needed.
#[tokio::test]
#[serial]
async fn production_needs_no_site_url_without_reset_nor_admin() {
    assert!(
        !refused(builder(false).await)
            .await
            .contains(&"SiteUrl".to_string())
    );
}

// In debug the request's Host stays the fallback: nothing to configure locally.
#[tokio::test]
#[serial]
async fn debug_boots_without_site_url() {
    let b = builder(true)
        .await
        .with_password_reset(|pr| pr)
        .with_admin(|a| a);
    let components = refused(b).await;
    assert!(
        !components.contains(&"SiteUrl".to_string()),
        "{components:?}"
    );
}

// The boot report follows the configured language, messages and suggestions alike.
#[tokio::test]
#[serial]
async fn the_boot_report_follows_the_language() {
    use runique::utils::trad::{Lang, current_lang, set_lang};
    let before = current_lang();
    set_lang(Lang::Fr);
    let report = builder(false)
        .await
        .with_password_reset(|pr| pr)
        .build()
        .await
        .err()
        .map(|e| e.to_string());
    set_lang(before);
    let report = report.expect("refused");
    assert!(report.contains("SITE_URL n'est pas défini"), "{report}");
    assert!(report.contains("Définissez SITE_URL"), "{report}");
}
