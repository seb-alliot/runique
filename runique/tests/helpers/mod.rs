//! Utilitaires partagés pour tous les tests.
//!
//! | Module    | Usage                                                        |
//! | --------- | ------------------------------------------------------------ |
//! | `server`  | Démarrer le moteur et le routeur de test                     |
//! | `request` | Builders oneshot : `get`, `post`, `delete`, …               |
//! | `assert`  | Assertions HTTP : `assert_status`, `assert_has_header`, …   |
//! | `db`      | SQLite en mémoire : `fresh_db()`, `exec()`, `count()`, …    |

pub mod assert;
pub mod db;
pub mod request;
pub mod server;
