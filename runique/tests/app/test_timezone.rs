//! `TZ` in the `.env`: the time zone of the site. A name that is no zone
//! stops the start instead of leaving every date as stored without a word.
use axum::Router;
use runique::app::RuniqueApp;
use runique::config::RuniqueConfig;
use sea_orm::Database;
use serial_test::serial;

/// The refusal of an application whose time zone is `zone`, or none.
async fn started_in(zone: &str) -> Option<String> {
    let mut config = RuniqueConfig::from_env();
    config.dev = false;
    config.server.secret_key = "a-long-enough-test-secret-key-for-production-0123456789".into();
    config.timezone = zone.to_string();
    RuniqueApp::builder(config)
        .with_database(Database::connect("sqlite::memory:").await.unwrap())
        .routes(Router::new())
        .static_files(|s| s.enabled(false))
        .middleware(|m| m.with_session_store(tower_sessions::MemoryStore::default()))
        .build()
        .await
        .err()
        .map(|e| e.to_string())
}

#[tokio::test]
#[serial]
async fn a_known_time_zone_lets_the_application_start() {
    for zone in ["UTC", "Europe/Paris", "America/Toronto", "Asia/Tokyo"] {
        assert_eq!(started_in(zone).await, None, "{zone}");
    }
}

// The name is given back in the refusal, as it was written.
#[tokio::test]
#[serial]
async fn a_name_that_is_no_time_zone_stops_the_start() {
    for zone in ["Europe/Pari", "Paris", "+01:00", "", "Europe/Paris "] {
        let refusal = started_in(zone)
            .await
            .unwrap_or_else(|| panic!("{zone:?} was accepted"));
        assert!(
            refusal.contains(&format!("TZ={zone}")),
            "{zone:?}: {refusal}"
        );
    }
}
