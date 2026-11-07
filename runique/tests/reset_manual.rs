//! Alone in its process: the app's `build()` sets the password config to
//! `Manual`, whose algorithm the built-in reset must hash with.
#[allow(dead_code)]
#[path = "helpers/mod.rs"]
mod helpers;
#[path = "reset_modes/support.rs"]
mod support;

use runique::db::ADb;
use runique::utils::password::{Manual, PasswordConfig, PasswordService, password_get};
use runique::utils::reset_token;
use std::time::Duration;

#[tokio::test]
async fn manual_password_reset_hashes_with_its_algorithm() {
    let config = PasswordConfig::manual(Manual::Argon2);
    let (base, db) = support::spawn(config.clone()).await;
    assert!(
        matches!(password_get(), PasswordConfig::Manual(_)),
        "the app's build must be the first to set the global config"
    );
    let adb = ADb::from_connection(db.clone());
    let token = reset_token::generate(&adb, helpers::pk::pk(1), Duration::from_secs(600))
        .await
        .unwrap();

    support::submit_reset(&base, &token).await;

    let stored = support::stored_password(&db).await;
    assert_ne!(stored, support::NEW_PASSWORD, "never stored in plaintext");
    assert_ne!(
        stored,
        support::SEEDED_HASH,
        "the reset wrote a new password"
    );
    assert!(
        PasswordService::new(config).verify(support::NEW_PASSWORD, &stored),
        "hashed by the Manual algorithm"
    );
}
