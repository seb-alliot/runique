//! Business-logic test builder (`test-utils` feature): each test runs against
//! a real database inside a transaction that is always rolled back, with the
//! SQL it executed printed alongside its verdict.
pub mod logic;

pub use logic::struct_test::{FormatResult, QueryTrace, Reason, TraceSink};
pub use logic::struct_test::{RuniqueTest, SetupError};
pub use logic::transaction_test::TestTransaction;
