//! Alone in its process: the app's `build()` sets the password config to
//! `Delegated`, which the built-in reset must follow.
#[allow(dead_code)]
#[path = "helpers/mod.rs"]
mod helpers;
#[path = "reset_modes/support.rs"]
mod support;

use runique::db::ADb;
use runique::utils::password::{External, PasswordConfig, password_get};
use runique::utils::reset_token;
use std::time::Duration;

#[tokio::test]
async fn delegated_password_reset_never_stores_a_local_password() {
    let (base, db) = support::spawn(PasswordConfig::Delegated(External::GoogleOAuth)).await;
    assert!(
        matches!(password_get(), PasswordConfig::Delegated(_)),
        "the app's build must be the first to set the global config"
    );
    let adb = ADb::from_connection(db.clone());
    let token = reset_token::generate(&adb, helpers::pk::pk(1), Duration::from_secs(600))
        .await
        .unwrap();

    support::submit_reset(&base, &token).await;

    assert_eq!(
        support::stored_password(&db).await,
        support::SEEDED_HASH,
        "Delegated mode must not write a local password during reset"
    );
    assert!(
        reset_token::peek(&adb, &token).await,
        "a delegated password reset must not consume the usable reset link"
    );
}
