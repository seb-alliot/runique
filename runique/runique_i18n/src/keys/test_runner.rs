//! Keys of `test_runner`.
pub mod error {
    pub const BUILD_FAILED: &str = "test_runner.error.build_failed";
    pub const CARGO_SPAWN_FAILED: &str = "test_runner.error.cargo_spawn_failed";
    pub const CARGO_WAIT_FAILED: &str = "test_runner.error.cargo_wait_failed";
    pub const DIR_READ_FAILED: &str = "test_runner.error.dir_read_failed";
    pub const FILE_NOT_DECLARED: &str = "test_runner.error.file_not_declared";
    pub const FILE_NOT_DECLARED_WARNING: &str = "test_runner.error.file_not_declared_warning";
    pub const MODULE_NOT_DECLARED: &str = "test_runner.error.module_not_declared";
    pub const MODULE_NOT_GATED: &str = "test_runner.error.module_not_gated";
    pub const NO_CARGO_TOML: &str = "test_runner.error.no_cargo_toml";
    pub const NO_MOD_FILE: &str = "test_runner.error.no_mod_file";
    pub const NO_SUCH_TEST: &str = "test_runner.error.no_such_test";
    pub const NO_TEST_FILE: &str = "test_runner.error.no_test_file";
    pub const OUTPUT_FAILED: &str = "test_runner.error.output_failed";
    pub const TEST_UTILS_MISSING: &str = "test_runner.error.test_utils_missing";
    pub const TEST_UTILS_SHIPPED: &str = "test_runner.error.test_utils_shipped";
    pub const TESTS_FAILED: &str = "test_runner.error.tests_failed";
}
