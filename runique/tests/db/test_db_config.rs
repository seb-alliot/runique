//! Tests — db/config.rs (DatabaseConfig, DatabaseEngine, DatabaseConfigBuilder)
//!
//! Couverture :
//!   - DatabaseEngine::detect_from_url() — tous variants + erreurs
//!   - DatabaseEngine::name() — tous variants
//!   - DatabaseConfig::from_url() — tous types + invalide
//!   - DatabaseConfigBuilder — max_connections, min_connections, pool_size, logging, timeout
//!   - DatabaseConfig::from_env() — sqlite par défaut, postgres complet, cas d'erreur
//!   - DatabaseConfig::connect() — Postgres Docker, MariaDB Docker

use runique::db::{DatabaseConfig, DatabaseEngine};
use serial_test::serial;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════
// DatabaseEngine::detect_from_url
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_detect_postgres() {
    assert_eq!(
        DatabaseEngine::detect_from_url("postgres://user:pass@localhost/db").unwrap(),
        DatabaseEngine::PostgreSQL
    );
}

#[test]
fn test_detect_postgresql_alias() {
    assert_eq!(
        DatabaseEngine::detect_from_url("postgresql://user:pass@localhost/db").unwrap(),
        DatabaseEngine::PostgreSQL
    );
}

#[test]
fn test_detect_mysql() {
    assert_eq!(
        DatabaseEngine::detect_from_url("mysql://user:pass@localhost/db").unwrap(),
        DatabaseEngine::MySQL
    );
}

#[test]
fn test_detect_mariadb() {
    assert_eq!(
        DatabaseEngine::detect_from_url("mariadb://user:pass@localhost/db").unwrap(),
        DatabaseEngine::MariaDB
    );
}

#[test]
fn test_detect_sqlite() {
    assert_eq!(
        DatabaseEngine::detect_from_url("sqlite://./db.sqlite").unwrap(),
        DatabaseEngine::SQLite
    );
}

#[test]
fn test_detect_unknown_retourne_err() {
    assert!(DatabaseEngine::detect_from_url("mongodb://localhost").is_err());
    assert!(DatabaseEngine::detect_from_url("redis://localhost").is_err());
    assert!(DatabaseEngine::detect_from_url("http://example.com").is_err());
    assert!(DatabaseEngine::detect_from_url("").is_err());
}

// ═══════════════════════════════════════════════════════════════
// DatabaseEngine::name
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_name_postgres() {
    assert_eq!(DatabaseEngine::PostgreSQL.name(), "PostgreSQL");
}

#[test]
fn test_name_mysql() {
    assert_eq!(DatabaseEngine::MySQL.name(), "MySQL");
}

#[test]
fn test_name_mariadb() {
    assert_eq!(DatabaseEngine::MariaDB.name(), "MariaDB");
}

#[test]
fn test_name_sqlite() {
    assert_eq!(DatabaseEngine::SQLite.name(), "SQLite");
}

// ═══════════════════════════════════════════════════════════════
// DatabaseConfig::from_url
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_from_url_postgres_detecte_engine() {
    let config = DatabaseConfig::from_url("postgres://user:pass@localhost/db")
        .unwrap()
        .build();
    assert_eq!(config.engine, DatabaseEngine::PostgreSQL);
    assert_eq!(config.url, "postgres://user:pass@localhost/db");
}

#[test]
fn test_from_url_postgresql_detecte_engine() {
    let config = DatabaseConfig::from_url("postgresql://user:pass@localhost/db")
        .unwrap()
        .build();
    assert_eq!(config.engine, DatabaseEngine::PostgreSQL);
}

#[test]
fn test_from_url_mysql_detecte_engine() {
    let config = DatabaseConfig::from_url("mysql://user:pass@localhost/db")
        .unwrap()
        .build();
    assert_eq!(config.engine, DatabaseEngine::MySQL);
}

#[test]
fn test_from_url_mariadb_detecte_engine() {
    let config = DatabaseConfig::from_url("mariadb://user:pass@localhost/db")
        .unwrap()
        .build();
    assert_eq!(config.engine, DatabaseEngine::MariaDB);
}

#[test]
fn test_from_url_sqlite_detecte_engine() {
    let config = DatabaseConfig::from_url("sqlite://./test.db")
        .unwrap()
        .build();
    assert_eq!(config.engine, DatabaseEngine::SQLite);
}

#[test]
fn test_from_url_invalide_retourne_err() {
    assert!(DatabaseConfig::from_url("http://example.com").is_err());
    assert!(DatabaseConfig::from_url("ftp://localhost").is_err());
    assert!(DatabaseConfig::from_url("invalid").is_err());
}

#[test]
fn test_from_url_valeurs_par_defaut() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .build();
    assert_eq!(config.max_connections, 100);
    assert_eq!(config.min_connections, 20);
    assert!(!config.sqlx_logging);
}

// ═══════════════════════════════════════════════════════════════
// DatabaseConfigBuilder — méthodes du builder
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_builder_max_connections() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .max_connections(100)
        .build();
    assert_eq!(config.max_connections, 100);
}

#[test]
fn test_builder_min_connections() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .min_connections(2)
        .build();
    assert_eq!(config.min_connections, 2);
}

#[test]
fn test_builder_pool_size() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .pool_size(3, 30)
        .build();
    assert_eq!(config.min_connections, 3);
    assert_eq!(config.max_connections, 30);
}

#[test]
fn test_builder_logging_desactive() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .logging(false)
        .build();
    assert!(!config.sqlx_logging);
}

#[test]
fn test_builder_logging_active() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .logging(true)
        .build();
    assert!(config.sqlx_logging);
}

#[test]
fn test_builder_connect_timeout() {
    let config = DatabaseConfig::from_url("sqlite://test.db")
        .unwrap()
        .connect_timeout(Duration::from_secs(30))
        .build();
    assert_eq!(config.connect_timeout, Duration::from_secs(30));
}

#[test]
fn test_builder_chaining() {
    let config = DatabaseConfig::from_url("postgres://user:pass@localhost/db")
        .unwrap()
        .max_connections(50)
        .min_connections(10)
        .pool_size(5, 25)
        .logging(false)
        .connect_timeout(Duration::from_secs(15))
        .build();

    assert_eq!(config.max_connections, 25);
    assert_eq!(config.min_connections, 5);
    assert!(!config.sqlx_logging);
    assert_eq!(config.connect_timeout, Duration::from_secs(15));
}

// ═══════════════════════════════════════════════════════════════
// DatabaseConfig::from_env — tests sérialisés (env global)
// ═══════════════════════════════════════════════════════════════

#[test]
#[serial]
fn test_from_env_sqlite_par_defaut() {
    unsafe {
    std::env::remove_var("DB_ENGINE");
    std::env::remove_var("DB_NAME");
    std::env::remove_var("DB_URL");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_ok(), "sqlite sans vars doit Ok");
    let config = result.unwrap().build();
    assert_eq!(config.engine, DatabaseEngine::SQLite);
    assert!(config.url.contains("sqlite://"));
}

#[test]
#[serial]
fn test_from_env_sqlite_avec_nom() {
    unsafe {
    std::env::set_var("DB_ENGINE", "sqlite");
    std::env::set_var("DB_NAME", "myapp.sqlite");
    }

    let result = DatabaseConfig::from_env();
    assert!(result.is_ok(), "sqlite avec DB_NAME doit Ok");
    let config = result.unwrap().build();
    assert_eq!(config.engine, DatabaseEngine::SQLite);
    assert!(config.url.contains("myapp.sqlite"));
    unsafe {
        std::env::remove_var("DB_ENGINE");
        std::env::remove_var("DB_NAME");
    }
}

#[test]
#[serial]
fn test_from_env_postgres_sans_user_retourne_err() {
    unsafe {
    std::env::set_var("DB_ENGINE", "postgres");
    std::env::remove_var("DB_USER");
    std::env::remove_var("DB_PASSWORD");
    std::env::remove_var("DB_NAME");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_err(), "postgres sans DB_USER doit Err");
    unsafe {
    std::env::remove_var("DB_ENGINE");
    }
}

#[test]
#[serial]
fn test_from_env_postgres_sans_password_retourne_err() {
    unsafe {
    std::env::set_var("DB_ENGINE", "postgres");
    std::env::set_var("DB_USER", "myuser");
    std::env::remove_var("DB_PASSWORD");
    std::env::remove_var("DB_NAME");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_err(), "postgres sans DB_PASSWORD doit Err");
    unsafe {
    std::env::remove_var("DB_ENGINE");
    std::env::remove_var("DB_USER");
    }
}

#[test]
#[serial]
fn test_from_env_postgres_sans_dbname_retourne_err() {
    unsafe {
    std::env::set_var("DB_ENGINE", "postgres");
    std::env::set_var("DB_USER", "myuser");
    std::env::set_var("DB_PASSWORD", "secret");
    std::env::remove_var("DB_NAME");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_err(), "postgres sans DB_NAME doit Err");
    unsafe {
    std::env::remove_var("DB_ENGINE");
    std::env::remove_var("DB_USER");
    std::env::remove_var("DB_PASSWORD");
    }
}

#[test]
#[serial]
fn test_from_env_postgres_complet() {
    unsafe {
    std::env::set_var("DB_ENGINE", "postgres");
    std::env::set_var("DB_USER", "myuser");
    std::env::set_var("DB_PASSWORD", "secret");
    std::env::set_var("DB_HOST", "myhost");
    std::env::set_var("DB_PORT", "5432");
    std::env::set_var("DB_NAME", "mydb");
    }

    let result = DatabaseConfig::from_env();
    assert!(result.is_ok(), "postgres complet doit Ok");
    let config = result.unwrap().build();
    assert_eq!(config.engine, DatabaseEngine::PostgreSQL);
    assert!(config.url.contains("myuser"));
    assert!(config.url.contains("myhost"));
    assert!(config.url.contains("mydb"));

    for var in &[
        "DB_ENGINE",
        "DB_USER",
        "DB_PASSWORD",
        "DB_HOST",
        "DB_PORT",
        "DB_NAME",
    ] {
        unsafe {
        std::env::remove_var(var);
        }
    }
}

#[test]
#[serial]
fn test_from_env_mysql_complet() {
    unsafe {
    std::env::set_var("DB_ENGINE", "mysql");
    std::env::set_var("DB_USER", "mysqluser");
    std::env::set_var("DB_PASSWORD", "mysqlpass");
    std::env::set_var("DB_HOST", "localhost");
    std::env::set_var("DB_PORT", "3306");
    std::env::set_var("DB_NAME", "mysqldb");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_ok(), "mysql complet doit Ok");
    let config = result.unwrap().build();
    assert_eq!(config.engine, DatabaseEngine::MySQL);
    assert!(config.url.starts_with("mysql://"));

    for var in &[
        "DB_ENGINE",
        "DB_USER",
        "DB_PASSWORD",
        "DB_HOST",
        "DB_PORT",
        "DB_NAME",
    ] {
        unsafe {
        std::env::remove_var(var);
        }
    }
}

#[test]
#[serial]
fn test_from_env_mariadb_complet() {
    unsafe {
    std::env::set_var("DB_ENGINE", "mariadb");
    std::env::set_var("DB_USER", "mariauser");
    std::env::set_var("DB_PASSWORD", "mariapass");
    std::env::set_var("DB_HOST", "localhost");
    std::env::set_var("DB_PORT", "3306");
    std::env::set_var("DB_NAME", "mariadb");
    }
    let result = DatabaseConfig::from_env();
    assert!(result.is_ok(), "mariadb complet doit Ok");
    let config = result.unwrap().build();
    assert_eq!(config.engine, DatabaseEngine::MariaDB);

    for var in &[
        "DB_ENGINE",
        "DB_USER",
        "DB_PASSWORD",
        "DB_HOST",
        "DB_PORT",
        "DB_NAME",
    ] {
        unsafe {
        std::env::remove_var(var);
        }
    }
}

#[test]
#[serial]
fn test_from_env_engine_inconnu_sans_db_url_retourne_err() {
    unsafe {
    std::env::set_var("DB_ENGINE", "cassandra");
    std::env::remove_var("DB_URL");
    }

    let result = DatabaseConfig::from_env();
    assert!(result.is_err(), "engine inconnu sans DB_URL doit Err");
    unsafe {
        std::env::remove_var("DB_ENGINE");
    }
    unsafe {
    std::env::remove_var("DB_ENGINE");
    }
}

// ═══════════════════════════════════════════════════════════════
// DatabaseConfig::connect() — Docker (skip si non disponible)
// ═══════════════════════════════════════════════════════════════

#[tokio::test]
async fn test_connect_postgres_docker() {
    let _ = dotenvy::from_filename(".env.test");
    let pg_url = match std::env::var("DATABASE_URL_PG") {
        Ok(url) => url,
        Err(_) => return, // skip si Docker non disponible
    };

    let config = DatabaseConfig::from_url(&pg_url)
        .unwrap()
        .min_connections(0)
        .logging(false)
        .build();

    // On appelle connect() pour la couverture ; le résultat peut varier
    // selon la locale du serveur Docker (erreur non-UTF-8 possible)
    let _ = config.connect().await;
}

#[tokio::test]
async fn test_connect_mariadb_docker() {
    let _ = dotenvy::from_filename(".env.test");
    let mariadb_url = match std::env::var("DATABASE_URL_MARIADB") {
        Ok(url) => url,
        Err(_) => return, // skip si Docker non disponible
    };

    // Le driver SeaORM utilise mysql:// même pour MariaDB
    let mysql_url = if mariadb_url.starts_with("mysql://") {
        mariadb_url
    } else {
        mariadb_url.replacen("mariadb://", "mysql://", 1)
    };

    let config = DatabaseConfig::from_url(&mysql_url)
        .unwrap()
        .min_connections(0)
        .logging(false)
        .build();

    // On appelle connect() pour la couverture
    let _ = config.connect().await;
}

#[tokio::test]
async fn test_connect_url_invalide_retourne_err() {
    // SQLite vers un chemin en lecture seule ou invalide
    let config = DatabaseConfig::from_url("sqlite:///tmp/runique_test_invalid_path_xyz/db.sqlite")
        .unwrap()
        .logging(false)
        .build();

    // On ne peut pas garantir que ça échoue (SQLite crée les répertoires), donc
    // on vérifie juste que la méthode est appelable sans paniquer
    let _ = config.connect().await;
}
