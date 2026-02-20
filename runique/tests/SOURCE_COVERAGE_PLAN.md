# Source Coverage Plan (`runique/src`)

This file tracks test coverage goals by top-level module from `runique/src`.

## Scope

Top-level modules exported by `runique/src/lib.rs`:

- `app`
- `config`
- `context`
- `db` (feature `orm`)
- `engine`
- `flash`
- `forms`
- `macros`
- `migration`
- `admin`
- `errors`
- `middleware`
- `utils`

## Current baseline tests

- `tests/integration_tests.rs` (forms-heavy integration tests)
- `tests/test.rs` (migration parser / diff tests)
- `tests/source_coverage_smoke.rs` (cross-module smoke coverage)

## Next recommended dedicated suites

- `tests/admin_suite.rs`
- `tests/middleware_suite.rs`
- `tests/config_context_engine_suite.rs`
- `tests/macros_suite.rs`
- `tests/utils_suite.rs`
- `tests/db_suite.rs` (`#[cfg(feature = "orm")]`)

## Coverage policy

- Model constraints remain source of truth.
- Form validation can be stricter than model constraints, never looser.
- New features in `runique/src` should include at least:
  - one unit test in the touched module or an existing related suite,
  - one integration-level check when behavior crosses module boundaries.
