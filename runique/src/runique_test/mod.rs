//! Test builder for business logic (`test-utils` feature). Each test runs
//! against a real database inside a transaction that always gets rolled back,
//! and prints the SQL it ran next to its result.
pub mod logic;

pub use logic::builder_test::runique_test;
pub use logic::struct_test::{FormatResult, QueryTrace, Reason, TestFailure, TraceSink};
pub use logic::transaction_test::TestTransaction;
