//! The connection rule on real servers: one env file per line of the rule,
//! read the way a project's `.env` is, then a real connection.
//!
//! `db/config.rs` tests which address is retained, and the fallback on
//! SQLite. Only a server can refuse a password, take an account written with
//! special characters, or read `?dbname=`.
//!
//! Need `docker compose up -d` and `DATABASE_URL_PG` / `DATABASE_URL_MYSQL`
//! in `.env.test`; without them each test returns right away.
use super::helpers::Scratch;
use runique::db::ADb;
use runique::runique_test::{TestFailure, runique_test};
use serial_test::serial;
use url::Url;

/// This copy's own database for an engine, and the parts of its address.
struct Server {
    url: Url,
    engine: &'static str,
}

impl Server {
    async fn of(url_key: &str, engine: &'static str) -> Option<Self> {
        let url = crate::helpers::db_isolation::isolated_url(url_key).await?;
        Some(Self {
            url: Url::parse(&url).expect("the test database address"),
            engine,
        })
    }

    fn password(&self) -> &str {
        self.url.password().expect("a password")
    }

    fn name(&self) -> &str {
        self.url.path().trim_start_matches('/')
    }

    /// The `DB_*` keys naming this database, minus `without`.
    fn keys(&self, without: &[&str]) -> Vec<String> {
        [
            ("DB_ENGINE", self.engine.to_string()),
            ("DB_USER", self.url.username().to_string()),
            ("DB_PASSWORD", self.password().to_string()),
            ("DB_HOST", self.url.host_str().expect("a host").to_string()),
            ("DB_PORT", self.url.port().expect("a port").to_string()),
            ("DB_NAME", self.name().to_string()),
        ]
        .into_iter()
        .filter(|(key, _)| !without.contains(key))
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
    }

    /// `DATABASE_URL=` with this address once `change` went over it.
    fn address(&self, change: impl FnOnce(&mut Url)) -> String {
        let mut url = self.url.clone();
        change(&mut url);
        format!("DATABASE_URL={url}")
    }
}

const WRONG_PASSWORD: &str = "wrong-Pw-0";

/// Connects the way a test of a project does, on the env file `lines` make.
async fn connect(case: &str, lines: Vec<String>) -> Result<(), TestFailure> {
    let scratch = Scratch::new(case);
    runique_test::<ADb>(&scratch.env_file(&lines), async |_db| Ok(())).await
}

fn with(first: String, mut rest: Vec<String>) -> Vec<String> {
    rest.insert(0, first);
    rest
}

/// Every line of the rule a server can answer.
async fn the_rule_on(server: &Server, tag: &str) {
    let case = |name: &str| format!("rule_{tag}_{name}");

    // The address alone, then the keys alone.
    let outcome = connect(&case("url"), vec![server.address(|_| {})]).await;
    assert!(outcome.is_ok(), "the address alone: {outcome:?}");
    let outcome = connect(&case("keys"), server.keys(&[])).await;
    assert!(outcome.is_ok(), "the keys alone: {outcome:?}");

    // Case A: the server refuses the password of the address, the keys answer.
    let refused = server.address(|url| url.set_password(Some(WRONG_PASSWORD)).unwrap());
    let outcome = connect(&case("refused"), with(refused.clone(), server.keys(&[]))).await;
    assert!(
        outcome.is_ok(),
        "a refused password, then the keys: {outcome:?}"
    );

    // Case A: nothing listens at the address, the keys answer.
    let silent = server.address(|url| url.set_port(Some(1)).unwrap());
    let outcome = connect(&case("silent"), with(silent, server.keys(&[]))).await;
    assert!(
        outcome.is_ok(),
        "nothing listening, then the keys: {outcome:?}"
    );

    // Case A, keys that are not enough: blocked, naming what misses, and
    // neither password comes out.
    let failure = connect(
        &case("partial"),
        with(refused.clone(), server.keys(&["DB_PASSWORD"])),
    )
    .await
    .expect_err("partial keys can't take over");
    assert!(
        failure.message.contains("DB_PASSWORD"),
        "{}",
        failure.message
    );
    assert!(
        !failure.message.contains(WRONG_PASSWORD),
        "{}",
        failure.message
    );

    // Case A, no key: the refusal is the error.
    let failure = connect(&case("alone"), vec![refused])
        .await
        .expect_err("a refused password alone");
    assert!(
        !failure.message.contains(WRONG_PASSWORD),
        "{}",
        failure.message
    );
    assert!(
        !failure.message.contains(server.password()),
        "{}",
        failure.message
    );

    // Line 2: an address that names no database gives way to the keys.
    let nameless = server.address(|url| url.set_path(""));
    let outcome = connect(&case("nameless"), with(nameless.clone(), server.keys(&[]))).await;
    assert!(
        outcome.is_ok(),
        "no database name, then the keys: {outcome:?}"
    );

    // Line 4a: the same address alone is refused before any connection.
    let failure = connect(&case("nameless_alone"), vec![nameless])
        .await
        .expect_err("no database name and no key");
    assert!(
        !failure.message.contains(server.password()),
        "{}",
        failure.message
    );

    // Keys without the engine: refused, SQLite is not taken in its place.
    let failure = connect(&case("no_engine"), server.keys(&["DB_ENGINE"]))
        .await
        .expect_err("keys without an engine");
    assert!(
        !failure.message.contains(server.password()),
        "{}",
        failure.message
    );
}

#[tokio::test]
#[serial]
#[cfg(feature = "postgres")]
async fn the_connection_rule_holds_on_postgres() {
    let Some(server) = Server::of("DATABASE_URL_PG", "postgres").await else {
        return;
    };
    the_rule_on(&server, "pg").await;

    // PostgreSQL also takes the database as a parameter: a complete address.
    let name = server.name().to_string();
    let by_parameter = server.address(|url| {
        url.set_path("");
        url.query_pairs_mut().append_pair("dbname", &name);
    });
    let outcome = connect("rule_pg_dbname", vec![by_parameter]).await;
    assert!(outcome.is_ok(), "the database as a parameter: {outcome:?}");
}

#[tokio::test]
#[serial]
#[cfg(feature = "mysql")]
async fn the_connection_rule_holds_on_mysql() {
    let Some(server) = Server::of("DATABASE_URL_MYSQL", "mysql").await else {
        return;
    };
    the_rule_on(&server, "mysql").await;
}

// An account whose name and password hold what cuts an address (`@`, `/`,
// `:`, `#`, `?`), written as they are in the keys: Runique encodes them, the
// server recognises the account.
#[tokio::test]
#[serial]
#[cfg(feature = "postgres")]
async fn an_account_with_special_characters_connects_by_the_keys() {
    use runique::sea_orm::ConnectionTrait;
    const ROLE: &str = "rq@spe/cial";
    const PASSWORD: &str = "p@ss/w:rd#1?";

    let Some(server) = Server::of("DATABASE_URL_PG", "postgres").await else {
        return;
    };
    let Some(admin) = crate::helpers::db_postgres::connect().await else {
        return;
    };
    let drop_role = format!("DROP ROLE IF EXISTS \"{ROLE}\"");
    admin
        .execute_unprepared(&drop_role)
        .await
        .expect("drop role");
    admin
        .execute_unprepared(&format!(
            "CREATE ROLE \"{ROLE}\" LOGIN PASSWORD '{PASSWORD}'"
        ))
        .await
        .expect("create role");

    // Quoted for the env file reader, which stops a bare value at `#`.
    let mut lines = server.keys(&["DB_USER", "DB_PASSWORD"]);
    lines.push(format!("DB_USER='{ROLE}'"));
    lines.push(format!("DB_PASSWORD='{PASSWORD}'"));
    let outcome = connect("rule_pg_special", lines).await;
    admin
        .execute_unprepared(&drop_role)
        .await
        .expect("drop role");

    assert!(outcome.is_ok(), "{outcome:?}");
}
