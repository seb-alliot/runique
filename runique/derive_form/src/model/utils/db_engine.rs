/// Database engine detected at compile time — via the Cargo feature forwarded
/// from `runique` in priority, `.env` as a fallback. Used by the proc-macro to
/// adapt generation according to the target engine.
#[derive(Debug, Clone, PartialEq)]
pub enum DbEngine {
    Postgres,
    Mysql,
    Sqlite,
    Unknown,
}

impl DbEngine {
    /// Detects the engine at compile time.
    ///
    /// Priority:
    /// 1. Cargo feature forwarded from `runique` (`postgres`/`mysql`/`sqlite`) —
    ///    the source of truth: it can't drift from what's actually compiled,
    ///    unlike `.env`, which can lie (wrong file, value forgotten after
    ///    switching engines).
    /// 2. `.env` fallback (`DB_ENGINE` then `DATABASE_URL`) — for a project whose
    ///    `Cargo.toml` doesn't yet forward the feature to `derive_form`.
    pub fn detect() -> Self {
        if cfg!(feature = "postgres") {
            return DbEngine::Postgres;
        }
        if cfg!(feature = "mysql") {
            return DbEngine::Mysql;
        }
        if cfg!(feature = "sqlite") {
            return DbEngine::Sqlite;
        }
        Self::detect_from_env()
    }

    /// Older mechanism, kept as a fallback until every project has migrated to
    /// the `runique` -> `derive_form` feature forwarding.
    fn detect_from_env() -> Self {
        // 1. CARGO_MANIFEST_DIR — path of the compiled crate (most reliable in proc-macro).
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let candidate = std::path::Path::new(&manifest_dir).join(".env");
            if candidate.exists() {
                let _ = dotenvy::from_path(&candidate);
            }
        }

        // 2. CWD then traversing tree up (root workspace fallback).
        if std::env::var("DATABASE_URL").is_err() && std::env::var("DB_ENGINE").is_err() {
            let _ = dotenvy::dotenv();
        }
        if std::env::var("DATABASE_URL").is_err() && std::env::var("DB_ENGINE").is_err() {
            let mut dir = std::env::current_dir().ok();
            for _ in 0..4 {
                if let Some(d) = dir {
                    let candidate = d.join(".env");
                    if candidate.exists() {
                        let _ = dotenvy::from_path(&candidate);
                        break;
                    }
                    dir = d.parent().map(|p| p.to_path_buf());
                } else {
                    break;
                }
            }
        }

        // 1. Explicit DB_ENGINE override
        if let Ok(engine) = std::env::var("DB_ENGINE") {
            match engine.to_ascii_lowercase().as_str() {
                "postgres" | "postgresql" => return DbEngine::Postgres,
                "mysql" | "mariadb" => return DbEngine::Mysql,
                "sqlite" => return DbEngine::Sqlite,
                _ => {}
            }
        }

        // 2. DATABASE_URL prefix
        if let Ok(url) = std::env::var("DATABASE_URL") {
            if url.starts_with("postgres://") || url.starts_with("postgresql://") {
                return DbEngine::Postgres;
            }
            if url.starts_with("mysql://") || url.starts_with("mariadb://") {
                return DbEngine::Mysql;
            }
            if url.starts_with("sqlite:") || url.starts_with("sqlite://") {
                return DbEngine::Sqlite;
            }
        }

        DbEngine::Unknown
    }

    pub fn is_postgres(&self) -> bool {
        matches!(self, DbEngine::Postgres)
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, DbEngine::Unknown)
    }
}

/// Refuses the types the compiled engine can't read back (see
/// `FormFieldKind::unsupported_on`): better a compile error than a runtime one
/// on the first read. Only the Cargo feature counts here, not `.env`, and
/// nothing is refused under `all-databases`, where no single engine is targeted.
pub fn check_engine_support(model: &crate::model::ast::ModelInput) -> syn::Result<()> {
    use crate::model::ast::FormFieldKind;
    use runique_dsl::types::Engine;

    if cfg!(feature = "all-databases") {
        return Ok(());
    }
    let (engine, engine_name) = if cfg!(feature = "postgres") {
        (Engine::Postgres, "Postgres")
    } else if cfg!(feature = "sqlite") {
        (Engine::Sqlite, "SQLite")
    } else {
        return Ok(());
    };
    for field in &model.fields {
        if !field.kind.unsupported_on().contains(&engine) {
            continue;
        }
        let instead = match field.kind {
            FormFieldKind::I8 => "`i16`",
            _ => "`bigint` (i64)",
        };
        let kind = format!("{:?}", field.kind).to_lowercase();
        return Err(syn::Error::new(
            field.name.span(),
            format!(
                "`{kind}` isn't supported on {engine_name}: the column can't be read back as a Rust `{kind}`. Use {instead} instead."
            ),
        ));
    }
    for e in &model.enums {
        if e.backing_type.unsupported_on().contains(&engine) {
            return Err(syn::Error::new(
                e.name.span(),
                format!(
                    "enum `{}` is `i8`, which isn't supported on {engine_name}: the value can't be read back as a Rust `i8`. Use `i16` instead.",
                    e.name
                ),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_engine_support;
    use crate::model::ast::ModelInput;

    fn check(field: &str) -> syn::Result<()> {
        let src = format!(r#"Case, table: "cases", pk: id => i32, {{ v: {field} [required], }}"#);
        check_engine_support(&syn::parse_str::<ModelInput>(&src).expect("parses"))
    }

    #[test]
    #[cfg(all(feature = "postgres", not(feature = "all-databases")))]
    fn postgres_refuses_what_it_cant_read_back() {
        for kind in ["i8", "u32", "u64"] {
            assert!(check(kind).is_err(), "{kind} must be refused on Postgres");
        }
        for kind in ["i16", "int", "bigint", "timestamp_tz"] {
            assert!(check(kind).is_ok(), "{kind} must be accepted on Postgres");
        }
    }

    #[test]
    #[cfg(all(feature = "sqlite", not(feature = "all-databases")))]
    fn sqlite_refuses_only_u64() {
        assert!(check("u64").is_err());
        for kind in ["i8", "u32", "int"] {
            assert!(check(kind).is_ok(), "{kind} must be accepted on SQLite");
        }
    }

    // The `.env` fallback, for a project that forwards no engine feature.
    // Process-wide variables: these tests take turns.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn detect_with(db_engine: Option<&str>, url: Option<&str>) -> super::DbEngine {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        // SAFETY: serialized by ENV_LOCK; no other derive_form test reads these.
        unsafe {
            match db_engine {
                Some(v) => std::env::set_var("DB_ENGINE", v),
                None => std::env::remove_var("DB_ENGINE"),
            }
            match url {
                Some(v) => std::env::set_var("DATABASE_URL", v),
                None => std::env::remove_var("DATABASE_URL"),
            }
        }
        let engine = super::DbEngine::detect_from_env();
        unsafe {
            std::env::remove_var("DB_ENGINE");
            std::env::remove_var("DATABASE_URL");
        }
        engine
    }

    #[test]
    fn db_engine_names_the_engine() {
        use super::DbEngine::*;
        for (value, engine) in [
            ("postgres", Postgres),
            ("PostgreSQL", Postgres),
            ("mysql", Mysql),
            ("mariadb", Mysql),
            ("sqlite", Sqlite),
        ] {
            assert_eq!(detect_with(Some(value), None), engine, "{value}");
        }
    }

    #[test]
    fn otherwise_the_url_scheme_names_it() {
        use super::DbEngine::*;
        for (url, engine) in [
            ("postgres://u@h/db", Postgres),
            ("postgresql://u@h/db", Postgres),
            ("mysql://u@h/db", Mysql),
            ("mariadb://u@h/db", Mysql),
            ("sqlite:db.sqlite", Sqlite),
            ("sqlite://db.sqlite", Sqlite),
        ] {
            assert_eq!(detect_with(Some("oracle"), Some(url)), engine, "{url}");
        }
    }

    #[test]
    fn engine_predicates() {
        use super::DbEngine::*;
        assert!(Postgres.is_postgres());
        assert!(!Mysql.is_postgres() && !Sqlite.is_postgres() && !Unknown.is_postgres());
        assert!(Unknown.is_unknown());
        assert!(!Postgres.is_unknown());
    }

    #[test]
    #[cfg(any(
        feature = "all-databases",
        not(any(feature = "postgres", feature = "sqlite"))
    ))]
    fn nothing_refused_without_a_single_strict_engine() {
        for kind in ["i8", "u32", "u64"] {
            assert!(check(kind).is_ok(), "{kind} must be accepted");
        }
    }
}
