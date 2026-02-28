//! # Suite de tests — runique
//!
//! ## Conventions de nommage
//! Tous les fichiers de test portent le préfixe `test_` :
//! - `test_csrf.rs`, `test_validator.rs`, `test_paths.rs`, …
//!
//! ## Types de tests
//! | Type          | Attribut           | Quand l'utiliser                          |
//! | ------------- | ------------------ | ----------------------------------------- |
//! | Unitaire      | `#[test]`          | Tester une fonction/struct isolée         |
//! | Async oneshot | `#[tokio::test]`   | Tester un handler axum sans session       |
//! | Intégration   | `#[tokio::test]`   | Flux multi-requêtes avec cookie/session   |
//!
//! ## Ajouter un nouveau test
//! 1. Créer `<module>/test_<sujet>.rs`
//! 2. Le déclarer dans `<module>/mod.rs`
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
//! - [`helpers::db`]      — SQLite en mémoire : `fresh_db()`, `fresh_db_with_schema()`, `exec()`, `count()`, …

pub mod admin;
pub mod auth;
pub mod config;
pub mod db;
pub mod errors;
pub mod formulaire;
pub mod helpers;
pub mod middleware;
pub mod migration;
pub mod utils;
