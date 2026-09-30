//! Does each DSL type survive a write and a read, on each engine?
//!
//! The column is created the way a generated migration creates it: the CLI's
//! own parser decides the column type, then the matching sea-query method builds
//! it. The value goes in and comes back through the entity `model!{}` generates.
//! A mismatch between the two (say, an `i32` Rust field on a `SMALLINT` column)
//! only shows up here, on the engine that's strict about it.
//!
//! Postgres/MariaDB need `docker compose up -d` and `DATABASE_URL_PG` /
//! `DATABASE_URL_MARIADB` in `.env.test`; without them those tests return early.
use crate::helpers::{db_mariadb, db_postgres};
use runique::migration::utils::helpers::col_type_to_method;
use runique::migration::utils::parser_builder::parse_schema_from_source;
use runique::sea_orm::{
    ConnectionTrait, Database, DatabaseConnection,
    sea_query::{Alias, ColumnDef, Table},
};
use serial_test::serial;

/// One DSL type: a `model!{}` with a single required field `v` of that type,
/// the same declaration as text for the CLI's parser, and a round trip.
macro_rules! case {
    ($module:ident, $kind:ident, $table:literal, $value:expr) => {
        mod $module {
            use runique::prelude::*;

            model! {
                Case,
                table: $table,
                pk: id => i32,
                {
                    v: $kind [required],
                }
            }

            pub const KIND: &str = stringify!($kind);
            pub const TABLE: &str = $table;

            pub const SOURCE: &str = concat!(
                "model! { Case, table: ",
                stringify!($table),
                ", pk: id => i32, { v: ",
                stringify!($kind),
                " [required], } }"
            );

            /// Writes the value, reads it back, and checks it's unchanged.
            pub async fn round_trip(
                db: &runique::sea_orm::DatabaseConnection,
            ) -> Result<(), String> {
                let value = $value;
                let saved = ActiveModel {
                    v: Set(value.clone()),
                    ..Default::default()
                }
                .insert(db)
                .await
                .map_err(|e| format!("insert: {e}"))?;
                let read = Entity::find_by_id(saved.id)
                    .one(db)
                    .await
                    .map_err(|e| format!("read: {e}"))?
                    .ok_or("read: row not found")?;
                if read.v == value {
                    Ok(())
                } else {
                    Err(format!("read back {:?} instead of {:?}", read.v, value))
                }
            }
        }
    };
}

case!(t_int, int, "rq_types_int", 42);
case!(t_i8, i8, "rq_types_i8", 7);
case!(t_i16, i16, "rq_types_i16", 300);
case!(t_u32, u32, "rq_types_u32", 3_000_000_000);
case!(t_u64, u64, "rq_types_u64", 5);
case!(t_f32, f32, "rq_types_f32", 1.5);
case!(t_char, char, "rq_types_char", "a".to_string());
// Whole seconds: MariaDB's `TIMESTAMP` drops the fraction.
case!(
    t_timestamp_tz,
    timestamp_tz,
    "rq_types_timestamp_tz",
    chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap()
);
case!(t_blob, blob, "rq_types_blob", vec![0u8, 1, 127, 255]);
case!(
    t_var_binary,
    var_binary,
    "rq_types_var_binary",
    vec![0u8, 1, 127, 255]
);
case!(t_binary, binary, "rq_types_binary", vec![0u8, 1, 127, 255]);
case!(t_ip, ip, "rq_types_ip", "127.0.0.1".to_string());
case!(
    t_mac_address,
    mac_address,
    "rq_types_mac_address",
    "08:00:2b:01:02:03".to_string()
);

type RoundTrip =
    fn(
        &DatabaseConnection,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + '_>>;

fn cases() -> Vec<(&'static str, &'static str, &'static str, RoundTrip)> {
    macro_rules! entry {
        ($module:ident) => {
            ($module::KIND, $module::TABLE, $module::SOURCE, |db| {
                Box::pin($module::round_trip(db))
            })
        };
    }
    vec![
        entry!(t_int),
        entry!(t_i8),
        entry!(t_i16),
        entry!(t_u32),
        entry!(t_u64),
        entry!(t_f32),
        entry!(t_char),
        entry!(t_timestamp_tz),
        entry!(t_blob),
        entry!(t_var_binary),
        entry!(t_binary),
        entry!(t_ip),
        entry!(t_mac_address),
    ]
}

/// The column the CLI would generate for this declaration: its type name, and
/// the sea-query method a migration file would call for it.
fn cli_column(source: &str) -> (String, String) {
    let (_, schema) = parse_schema_from_source(source).expect("the CLI parses the model");
    let col_type = schema.columns[0].col_type.clone();
    let method = col_type_to_method(&col_type).to_string();
    (col_type, method)
}

/// Builds the column with the real sea-query method for `col_type`. When the
/// CLI's method name doesn't exist in sea-query, the correct one is used here
/// so the round trip can still be tested; the mismatch is reported apart.
fn column_def(col_type: &str) -> Result<ColumnDef, String> {
    let mut column = ColumnDef::new(Alias::new("v"));
    match col_type {
        "TinyInteger" => column.tiny_integer(),
        "SmallInteger" => column.small_integer(),
        "Integer" => column.integer(),
        "BigInteger" => column.big_integer(),
        "Unsigned" => column.unsigned(),
        "BigUnsigned" => column.big_unsigned(),
        "Float" => column.float(),
        "Double" => column.double(),
        "String" => column.string(),
        "Char" => column.char(),
        "DateTime" => column.date_time(),
        "TimestampWithTimeZone" => column.timestamp_with_time_zone(),
        "Binary" => column.binary(),
        // The CLI writes `.var_binary()`, which sea-query doesn't have without a length.
        "VarBinary" => column.var_binary(255),
        "Blob" => column.blob(),
        other => return Err(format!("col_type {other} not handled by this test")),
    };
    column.not_null();
    Ok(column)
}

async fn run_all(db: &DatabaseConnection, engine: &str) -> Vec<String> {
    let mut report = Vec::new();
    let mut failures = Vec::new();
    // One table per type: Postgres caches prepared statements by their SQL
    // text, so reusing a table name with another column type trips the cache.
    for (kind, table, source, round_trip) in cases() {
        let (col_type, method) = cli_column(source);
        let drop = Table::drop()
            .table(Alias::new(table))
            .if_exists()
            .to_owned();
        db.execute(&drop).await.expect("drop");

        let outcome = match column_def(&col_type) {
            Err(e) => Err(e),
            Ok(mut column) => {
                let create = Table::create()
                    .table(Alias::new(table))
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(&mut column)
                    .to_owned();
                match db.execute(&create).await {
                    Err(e) => Err(format!("create table: {e}")),
                    Ok(_) => round_trip(db).await,
                }
            }
        };
        let verdict = match &outcome {
            Ok(()) => "ok".to_string(),
            Err(e) => format!("FAIL — {e}"),
        };
        report.push(format!(
            "{engine:8} {kind:14} → {col_type:22} .{method:28} {verdict}"
        ));
        if outcome.is_err() {
            failures.push(kind.to_string());
        }
        db.execute(&drop).await.expect("drop after");
    }
    println!("\n{}", report.join("\n"));
    failures
}

/// Types that fail on each engine. The tests check the failures are exactly
/// these: a new failure is a regression, and a type that starts passing means
/// the list is out of date.
/// These are the types `model!{}` refuses at compile time when a single engine
/// is compiled; they only get this far because the suite runs under
/// `all-databases`, where no engine is refused.
/// - `i8`: Postgres has no one-byte integer; sqlx sends an `i8` as `"char"`.
/// - `u32`: sea-query's `unsigned()` is `BIGINT` on Postgres, read back as `i32`.
/// - `u64`: sqlx only handles it on MySQL/MariaDB.
const KNOWN_FAILING_SQLITE: &[&str] = &["u64"];
const KNOWN_FAILING_POSTGRES: &[&str] = &["i8", "u32", "u64"];
/// - `blob`, `binary` (MariaDB): bugs of the CLI's own type table, fixed when
///   it moves onto `runique_dsl` (chantier C): `blob` becomes a `Binary`
///   column, and `.binary()` without a length is a one-byte `BINARY(1)`. The
///   CLI also writes `.var_binary()` without a length, which sea-query lacks
///   (this test builds that one with 255).
const KNOWN_FAILING_MARIADB: &[&str] = &["blob", "binary"];

fn check(engine: &str, failures: Vec<String>, known: &[&str]) {
    assert_eq!(
        failures, known,
        "{engine}: the failing types changed (see the table above)"
    );
}

#[tokio::test]
async fn dsl_types_round_trip_on_sqlite() {
    let db = Database::connect("sqlite::memory:").await.expect("sqlite");
    check("SQLite", run_all(&db, "sqlite").await, KNOWN_FAILING_SQLITE);
}

#[tokio::test]
#[serial]
async fn dsl_types_round_trip_on_postgres() {
    let Some(db) = db_postgres::connect().await else {
        return;
    };
    check(
        "Postgres",
        run_all(&db, "postgres").await,
        KNOWN_FAILING_POSTGRES,
    );
}

#[tokio::test]
#[serial]
async fn dsl_types_round_trip_on_mariadb() {
    let Some(db) = db_mariadb::connect().await else {
        return;
    };
    check(
        "MariaDB",
        run_all(&db, "mariadb").await,
        KNOWN_FAILING_MARIADB,
    );
}
