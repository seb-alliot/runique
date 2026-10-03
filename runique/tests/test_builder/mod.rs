//! Tests of the `runique_test` builder itself (`test-utils` feature): each one
//! tries to make it lie (a green test with data left behind) or hang.
#[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
pub mod helpers;
// Each test needs its engine's driver (`postgres` / `mysql` feature).
#[cfg(any(feature = "postgres", feature = "mysql"))]
pub mod test_engines;
// Both run on a SQLite file: without the `sqlite` feature there's no driver.
#[cfg(feature = "sqlite")]
pub mod test_isolation;
#[cfg(feature = "sqlite")]
pub mod test_reporting;
