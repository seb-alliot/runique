//! Utilitaires partagés pour tous les tests.
//!
//! | Module        | Usage                                                        |
//! | ------------- | ------------------------------------------------------------ |
//! | `server`      | Démarrer le moteur et le routeur de test                     |
//! | `request`     | Builders oneshot : `get`, `post`, `delete`, …               |
//! | `assert`      | Assertions HTTP : `assert_status`, `assert_has_header`, …   |
//! | `db`          | SQLite en mémoire : `fresh_db()`, `exec()`, `count()`, …    |
//! | `db_postgres` | PostgreSQL Docker : `connect()`, `exec()`, `count()`, …     |
//! | `db_mariadb`  | MariaDB Docker : `connect()`, `exec()`, `count()`, …        |
//! | `tera`        | Appel direct de filtres/fonctions : `kwargs()`, `no_kwargs()` |

pub mod admin_server;
pub mod assert;
pub mod db;
pub mod db_mariadb;
pub mod db_postgres;
pub mod request;
pub mod server;
pub mod tera;
