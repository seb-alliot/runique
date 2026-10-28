/// The env file every test in this folder reads its database settings from,
/// one per engine the app is compiled for. The CI's workspace-wide features
/// turn all of them on at once: Postgres first, then MariaDB.
#[cfg(feature = "postgres")]
pub const ENV: &str = ".env";
#[cfg(all(feature = "mariadb", not(feature = "postgres")))]
pub const ENV: &str = ".env.mariadb";
#[cfg(all(feature = "mysql", not(feature = "postgres"), not(feature = "mariadb")))]
pub const ENV: &str = ".env.mysql";

mod blog;
mod contribution;
mod expected_failures;
mod form_save;
mod form_upload;
mod user;
