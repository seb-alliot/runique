//! Test builder for business logic (`test-utils` feature). Each test runs
//! against a real database inside a transaction that always gets rolled back,
//! and prints the SQL it ran next to its result.
pub mod logic;

pub use logic::builder_test::{TIMEOUT_KEY, runique_test};
pub use logic::expect_test::expect_db_error;
pub use logic::struct_test::{FormatResult, Issue, QueryTrace, Reason, TestFailure, TraceSink};
pub use logic::transaction_test::TestTransaction;
