//! Keys of `runique_test`.
pub mod error {
    pub const BAD_TIMEOUT: &str = "runique_test.error.bad_timeout";
    pub const BEGIN_FAILED: &str = "runique_test.error.begin_failed";
    pub const CLONE_OUTLIVED: &str = "runique_test.error.clone_outlived";
    pub const CONNECT_FAILED: &str = "runique_test.error.connect_failed";
    pub const ENV_LINE_MALFORMED: &str = "runique_test.error.env_line_malformed";
    pub const ENV_PARSE_FAILED: &str = "runique_test.error.env_parse_failed";
    pub const ENV_READ_FAILED: &str = "runique_test.error.env_read_failed";
    pub const NESTED: &str = "runique_test.error.nested";
    pub const NO_DB_CONFIG: &str = "runique_test.error.no_db_config";
    pub const NO_PANIC_MESSAGE: &str = "runique_test.error.no_panic_message";
    pub const NOT_A_TRANSACTION: &str = "runique_test.error.not_a_transaction";
    pub const NOT_REFUSED: &str = "runique_test.error.not_refused";
    pub const PANICKED: &str = "runique_test.error.panicked";
    pub const QUERY_FAILED: &str = "runique_test.error.query_failed";
    pub const ROLLBACK_FAILED: &str = "runique_test.error.rollback_failed";
    pub const ROLLBACK_TIMED_OUT: &str = "runique_test.error.rollback_timed_out";
    pub const SWALLOWED_FAILURES: &str = "runique_test.error.swallowed_failures";
    pub const TEST_FAILED: &str = "runique_test.error.test_failed";
    pub const TIMED_OUT: &str = "runique_test.error.timed_out";
    pub const TRANSACTION_ENDED: &str = "runique_test.error.transaction_ended";
}
pub mod label {
    pub const EXPECTED: &str = "runique_test.label.expected";
    pub const IN_MEMORY: &str = "runique_test.label.in_memory";
}
