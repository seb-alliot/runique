//! The engine of the project, said twice: by its `.env`, which the command
//! connects with, and by the features of `runique` in its `Cargo.toml`,
//! which it is compiled with. The command is built for the three engines and
//! connects to any of them: it is the one that has to see them disagree.
use crate::db::DatabaseEngine;
use crate::migration::steps::generate::init_migrate::Engine;
use anyhow::{Result, bail};
use runique_i18n::tf;

const ENGINE_FEATURES: [&str; 3] = ["sqlite", "postgres", "mysql"];

/// Refuses an engine the project is not compiled for: `engine` is the one of
/// `.env`, `features` are those of `runique` in the project. With
/// `all-databases` the three are on, and `.env` alone decides.
pub(crate) fn check(engine: &DatabaseEngine, features: &[String]) -> Result<()> {
    let wanted = Engine::from(engine.clone()).feature();
    if features.iter().any(|feature| feature == wanted) {
        return Ok(());
    }
    let compiled: Vec<&str> = ENGINE_FEATURES
        .into_iter()
        .filter(|name| features.iter().any(|feature| feature == name))
        .collect();
    let compiled = compiled.join(", ");
    bail!(tf(
        runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
        &[engine.name(), compiled.as_str(), engine.name(), wanted]
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn features(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| name.to_string()).collect()
    }

    #[test]
    fn the_engine_of_the_env_passes_when_the_project_has_its_feature() {
        for (engine, feature) in [
            (DatabaseEngine::SQLite, "sqlite"),
            (DatabaseEngine::PostgreSQL, "postgres"),
            (DatabaseEngine::MySQL, "mysql"),
        ] {
            check(&engine, &features(&["default", "orm", feature])).unwrap();
        }
    }

    // `all-databases` turns the three on: whatever `.env` names is there.
    #[test]
    fn with_every_engine_compiled_the_env_decides() {
        let all = features(&["all-databases", "sqlite", "postgres", "mysql"]);
        for engine in [
            DatabaseEngine::SQLite,
            DatabaseEngine::PostgreSQL,
            DatabaseEngine::MySQL,
        ] {
            check(&engine, &all).unwrap();
        }
    }

    #[test]
    fn an_engine_the_project_is_not_compiled_for_is_refused() {
        let error = check(&DatabaseEngine::SQLite, &features(&["orm", "postgres"]))
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            tf(
                runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
                &["SQLite", "postgres", "SQLite", "sqlite"]
            )
        );
    }

    // Each engine against a project compiled for each of the two others.
    #[test]
    fn each_engine_is_refused_by_a_project_compiled_for_another() {
        let engines = [
            (DatabaseEngine::SQLite, "sqlite"),
            (DatabaseEngine::PostgreSQL, "postgres"),
            (DatabaseEngine::MySQL, "mysql"),
        ];
        for (asked, wanted) in &engines {
            for (_, compiled) in engines.iter().filter(|(_, other)| other != wanted) {
                let error = check(asked, &features(&["orm", *compiled]))
                    .unwrap_err()
                    .to_string();
                assert_eq!(
                    error,
                    tf(
                        runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
                        &[asked.name(), *compiled, asked.name(), *wanted]
                    ),
                    "{wanted} asked, {compiled} compiled"
                );
            }
        }
    }

    // No engine feature at all: nothing set in `.env` means SQLite, which
    // this project doesn't have either.
    #[test]
    fn a_project_without_any_engine_feature_is_refused() {
        let error = check(&DatabaseEngine::SQLite, &features(&["default", "orm"]))
            .unwrap_err()
            .to_string();
        assert_eq!(
            error,
            tf(
                runique_i18n::keys::makemigrations::error::ENGINE_NOT_IN_FEATURES,
                &["SQLite", "", "SQLite", "sqlite"]
            )
        );
    }

    // A feature whose name only holds an engine's is not that engine.
    #[test]
    fn a_feature_is_matched_by_its_whole_name() {
        assert!(check(&DatabaseEngine::SQLite, &features(&["sqlite-extra"])).is_err());
        assert!(check(&DatabaseEngine::PostgreSQL, &features(&["postgresql"])).is_err());
    }
}
