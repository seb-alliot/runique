//! # Suite de tests — runique
//!
//! All test files have the `test_` prefix:
//! - `test_csrf.rs`, `test_validator.rs`, `test_paths.rs`, …
//!
//! ## Types of tests
//! | Type          | Attribute          | When to use                              |
//! | ------------- | ------------------ | ---------------------------------------- |
//! | Unit          | `#[test]`          | Test an isolated function/struct         |
//! | Async oneshot | `#[tokio::test]`   | Test an axum handler without session     |
//! | Integration   | `#[tokio::test]`   | Multi-request flow with cookie/session   |
//!
//! ## Adding a new test
//! 1. Create `<module>/test_<subject>.rs`
//! 2. Declare it in `<module>/mod.rs`
//! 3. Utiliser les helpers disponibles :
//!
//! ```rust
//! use crate::helpers::{
//!     server::{build_engine, build_default_router},
//!     request,        // get(), post(), delete(), …
//!     assert,         // assert_status(), assert_has_header(), …
//! };
//! ```
//!
//! ## Helpers disponibles (`tests/helpers/`)
//! - [`helpers::server`]  — `build_engine()`, `build_default_router()`, `test_server_addr()`, `test_client()`
//! - [`helpers::request`] — builders oneshot : `get`, `post`, `post_with_header`, `delete`, …
//! - [`helpers::assert`]  — assertions HTTP : `assert_status`, `assert_has_header`, `assert_redirect`, …
//! - [`helpers::db`]      — In-memory SQLite: `fresh_db()`, `fresh_db_with_schema()`, `exec()`, `count()`, …

pub mod admin;
pub mod app;
pub mod auth;
pub mod config;
pub mod context;
pub mod db;
pub mod errors;
pub mod flash;
pub mod formulaire;
pub mod helpers;
pub mod macros;
pub mod middleware;
pub mod migration;
pub mod utils;
