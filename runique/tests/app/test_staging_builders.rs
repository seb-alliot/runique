//! Admin and core staging builders keep what they're given — written from
//! cargo-mutants survivors (2026-10-02): each could return a default staging,
//! dropping the rest of the configuration, without a test failing.
use axum::Router;
use axum::routing::get;
use runique::admin::resource::CrudOperation;
use runique::app::RuniqueApp;
use runique::app::staging::{AdminStaging, CoreStaging};
use runique::auth::LoginGuard;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;

#[test]
fn admin_staging_builders_keep_every_setting() {
    let a = AdminStaging::new()
        .site_title("Back-office")
        .no_robots_txt()
        .sitemap("https://site.example/sitemap.xml")
        .page_size(7)
        .view_site_url("https://site.example")
        .resource_order(["users", "posts"])
        .with_login_guard(LoginGuard::new().max_attempts(3))
        .templates(|t| t.with_list("theme/list.html"))
        .extra_routes(vec![(
            "report",
            "posts",
            CrudOperation::View,
            get(|| async { "r" }),
        )]);

    assert_eq!(
        a.config.site_title, "Back-office",
        "earlier settings survive"
    );
    assert!(!a.robots_txt);
    assert_eq!(
        a.sitemap_url.as_deref(),
        Some("https://site.example/sitemap.xml")
    );
    assert_eq!(a.config.page_size, 7);
    assert_eq!(a.config.view_site_href(), "https://site.example");
    assert_eq!(a.config.resource_order, ["users", "posts"]);
    assert_eq!(
        a.config.login_guard.as_ref().map(|g| g.max_attempts),
        Some(3)
    );
    assert_eq!(a.config.templates.list.resolve(), "theme/list.html");
    let route = &a.extra_routes[0];
    assert_eq!(route.path, "/report", "a leading slash is added");
    assert_eq!(
        (route.resource.as_str(), route.operation),
        ("posts", CrudOperation::View)
    );
}

#[derive(Debug, PartialEq)]
struct SearchClient(u8);

#[tokio::test]
#[serial]
async fn an_extra_db_is_reachable_from_the_engine() {
    let mut config = RuniqueConfig::from_env();
    config.debug = true;
    let app = RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .core(|c| c.with_extra_db(SearchClient(7)))
        .routes(Router::new().route("/", get(|| async { "ok" })))
        .static_files(|s| s.enabled(false))
        .build()
        .await
        .unwrap();
    assert_eq!(
        app.engine.extension::<SearchClient>().as_deref(),
        Some(&SearchClient(7))
    );
}

#[test]
fn a_database_config_makes_the_core_ready() {
    assert!(CoreStaging::new().validate().is_err(), "no database at all");
    let db = runique::db::DatabaseConfig::from_url("sqlite://unused.db")
        .unwrap()
        .build();
    let core = CoreStaging::new().with_database_config(db);
    assert!(core.validate().is_ok());
    assert!(core.is_ready());
}
