# Testing a Runique Application

Runique tests your **business logic against a real database**: each test runs inside a transaction that is always rolled back, and prints the SQL it ran next to its result. That's the `runique_test` builder (`test-utils` feature) and the `runique test` command.

---

## 1. Setup

`test-utils` goes under `[dev-dependencies]` **only**: it compiles code that must never reach a release build.

```toml
# Cargo.toml
[dependencies]
runique = { version = "3.0.0", features = ["postgres"] }

[dev-dependencies]
runique = { version = "3.0.0", features = ["test-utils"] }
```

Tests live in `src/runique_test/`, one file per domain, declared behind `cfg(test)`:

```rust
// src/main.rs
#[cfg(test)]
mod runique_test;
```

```rust
// src/runique_test/mod.rs
/// The env file every test in this folder reads its database settings from.
pub const ENV: &str = ".env";

mod blog;
mod user;
```

The env file has to name its database (`DATABASE_URL` or `DB_ENGINE`): otherwise the test is refused, rather than a local SQLite file being quietly created. Nothing is read from the shell's environment, where `DATABASE_URL` could point anywhere.

---

## 2. Writing a test

A test is a `#[tokio::test]` returning `Result<(), TestFailure>`, whose last expression is `runique_test`:

```rust
// src/runique_test/blog.rs
use crate::backend::blog::get_article;
use crate::entities::blog;
use runique::prelude::*;
use runique::runique_test::{TestFailure, runique_test};
use sea_orm::DbErr;

#[tokio::test]
async fn created_article_is_found_by_id() -> Result<(), TestFailure> {
    runique_test::<ADb>(super::ENV, async |db| {
        let article = blog::ActiveModel {
            title: Set("Rolled back at the end".to_string()),
            ..Default::default()
        }
        .insert(db)
        .await?;

        get_article(db, article.id)
            .await
            .map(|_| ())
            .ok_or_else(|| DbErr::Custom("the article should be found by its id".into()))
    })
    .await
}
```

`db` is an `&ADb`: the same type as `request.engine.db`, so your business functions run unchanged. Whatever happens, the transaction is rolled back.

### What fails a test

- the handler returns `Err`, panics, or runs past the time limit (`RUNIQUE_TEST_TIMEOUT` in the env file, 30 s by default) — `runique_test` never panics, it returns `Err(TestFailure)` with its message;
- **a query failed while the handler returned `Ok`**: an error got swallowed somewhere (`.ok()`, `unwrap_or_default()`…);
- the transaction was ended from inside the test (a raw `COMMIT`, or DDL on MariaDB/MySQL, which commits implicitly).

### Expected refusals

A refusal you *want* (unique constraint, foreign key, `NOT NULL`) goes through `expect_db_error`: it runs in a savepoint — on Postgres a failed statement outside one spoils the whole transaction — and is marked as expected in the trace.

```rust
use runique::runique_test::expect_db_error;

let refused = expect_db_error(db, async |sp| {
    contribution(deleted_user_id).insert(sp).await
})
.await?; // Err if the database accepted it
```

---

## 3. Running the tests

```bash
runique test                    # every test of src/runique_test/
runique test blog               # the tests of src/runique_test/blog.rs
runique test blog created_article_is_found_by_id   # a single test
```

`runique test` first checks the setup — `test-utils` only in `[dev-dependencies]` (refused when it reaches a release build through `[dependencies]`, `[workspace.dependencies]` or a `[features]` entry), the `runique_test` module behind `cfg(test)`, every file declared in `mod.rs` — then runs `cargo test` one test at a time, with its output shown. Plain `cargo test` works too: tests are serialized within the process.

Another engine (MongoDB…) can plug in by implementing the `TestTransaction` trait; `ADb` (SeaORM) is the implementation provided.

---

## 4. Boot checks

`RuniqueApp::builder(config).build().await` checks the configuration before starting.

**In every mode:**

- **Database**: no `.with_database(...)` / `.with_database_config(...)`;
- **AdminPanel**: an empty admin prefix, or an `extra_routes` entry naming a resource that isn't registered;
- **static_cache** / **media_cache**: a cache value that isn't a valid HTTP header;
- **MediaRoot**: a media folder that can't be created;
- **CORS**: `any_origin()` combined with `allow_credentials(true)`.

**Outside debug mode only** (`DEBUG=false`), what would be unsafe in production:

- **Security**: a weak `SECRET_KEY`;
- **PublicUrl**: the password reset or the admin enabled without `.with_public_url(...)`;
- **ACME**: `ACME_ENABLED` without `ACME_DOMAIN` or `ACME_EMAIL` (`acme` feature).

`build()` then returns a `BuildError` — `BuildErrorKind::CheckFailed(CheckReport)` for the checks above, CORS aside — displayed in the terminal with a suggestion for each problem.

---

← [**Examples**](/docs/en/exemple) | [**Troubleshooting**](/docs/en/installation/troubleshooting) →
