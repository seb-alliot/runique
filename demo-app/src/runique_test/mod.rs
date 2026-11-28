/// The env file every test in this folder reads its database settings from,
/// one per engine the app is compiled for. The CI's workspace-wide features
/// turn all of them on at once: Postgres first, then MySQL, then SQLite.
#[cfg(feature = "postgres")]
pub const ENV: &str = ".env";
#[cfg(all(feature = "mysql", not(feature = "postgres")))]
pub const ENV: &str = ".env.mysql";
#[cfg(all(feature = "sqlite", not(any(feature = "postgres", feature = "mysql"))))]
pub const ENV: &str = ".env.sqlite";

mod blog;
mod contribution;
mod expected_failures;
mod field_limits;
mod form_save;
mod form_upload;
mod user;
