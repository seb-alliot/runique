//! One Docker database per copy of the project.
//!
//! cargo-mutants runs several copies of the workspace at once against the same
//! Postgres and MariaDB containers, and the tests create and drop the same
//! tables (`eihwaz_*`, `items`, `rq_types_*`): left on one database, the copies
//! would run into each other and fail at random. Each copy gets its own
//! database instead, named after where this crate is built (`runique_test` →
//! `runique_test_1a2b3c4d`), so every run from a given copy reuses it.
//!
//! The MariaDB user needs the right to create those databases:
//! `GRANT ALL PRIVILEGES ON \`runique\_test\_%\`.* TO 'runique'@'%';`
use runique::sea_orm::{ConnectionTrait, Database};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::LazyLock;

/// Databases already created by this process: `key` → isolated URL.
static READY: LazyLock<tokio::sync::Mutex<HashMap<String, String>>> =
    LazyLock::new(Default::default);

/// The URL of this copy's own database for the engine `key` names
/// (`DATABASE_URL_PG`, `DATABASE_URL_MARIADB`), created if missing. `None`
/// when `key` isn't set: the tests needing it are skipped.
pub async fn isolated_url(key: &str) -> Option<String> {
    let _ = dotenvy::from_filename(".env.test");
    let base = std::env::var(key).ok()?;

    let mut ready = READY.lock().await;
    if let Some(url) = ready.get(key) {
        return Some(url.clone());
    }

    let (prefix, rest) = base.rsplit_once('/')?;
    let (db_name, query) = rest.split_once('?').map_or((rest, ""), |(d, q)| (d, q));
    let mut hasher = DefaultHasher::new();
    env!("CARGO_MANIFEST_DIR").hash(&mut hasher);
    // One database per primary key type: tables left by a run of another variant
    // keep foreign keys of the old type, which a recreated `eihwaz_users` can't satisfy.
    (cfg!(feature = "big-pk"), cfg!(feature = "pk-uuid")).hash(&mut hasher);
    let own_name = format!("{db_name}_{:08x}", hasher.finish() as u32);
    let own_url = if query.is_empty() {
        format!("{prefix}/{own_name}")
    } else {
        format!("{prefix}/{own_name}?{query}")
    };

    let admin = Database::connect(&base).await.unwrap_or_else(|e| {
        panic!("{key} is set but the container is unreachable: {e}\nRun `docker compose up -d`.")
    });
    let created = if base.starts_with("postgres") {
        admin
            .execute_unprepared(&format!("CREATE DATABASE \"{own_name}\""))
            .await
            .map(|_| ())
            .or_else(|e| {
                // Created by an earlier run of this copy.
                if e.to_string().contains("already exists") {
                    Ok(())
                } else {
                    Err(e)
                }
            })
    } else {
        admin
            .execute_unprepared(&format!("CREATE DATABASE IF NOT EXISTS `{own_name}`"))
            .await
            .map(|_| ())
    };
    if let Err(e) = created {
        panic!("could not create the test database {own_name} for {key}: {e}");
    }
    let _ = admin.close().await;

    ready.insert(key.to_string(), own_url.clone());
    Some(own_url)
}
