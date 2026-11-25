//! The type `Pk` stands for in the project the command runs in.
//!
//! `Pk` follows a feature of `runique` (`big-pk`, `pk-uuid`). The command is
//! built once and serves every project: its own features say nothing of the
//! project's, so it asks cargo which ones the project turns on.
use anyhow::{Result, anyhow, bail};
use runique_dsl::ast::PkType;
use runique_i18n::{t, tf};
use std::{fs, path::Path, process::Command};

/// Runs `read`, which reads the models, with `Pk` as the project has it.
/// Cargo is only asked when a file of `entities_path` writes `Pk`: a project
/// that spells its key types out never pays for the question.
pub(crate) fn with_project_pk<T>(
    entities_path: &str,
    read: impl FnOnce() -> Result<T>,
) -> Result<T> {
    if !alias_used(Path::new(entities_path)) {
        return read();
    }
    let ty = from_cargo()?;
    runique_dsl::with_project_pk(ty, read)
}

/// Whether a `.rs` file of the folder holds the word `Pk`. One in a comment
/// counts too: it costs a question to cargo, never a wrong type. A folder
/// that can't be read answers no: reading the models reports it.
fn alias_used(entities: &Path) -> bool {
    let Ok(files) = crate::migration::helpers::file::rust_files(entities) else {
        return false;
    };
    files
        .into_iter()
        .filter_map(|path| fs::read_to_string(path).ok())
        .any(|source| has_word(&source, "Pk"))
}

fn has_word(source: &str, word: &str) -> bool {
    source
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .any(|found| found == word)
}

/// Asks cargo for the project of the current folder.
pub(crate) fn from_cargo() -> Result<PkType> {
    let mut command = Command::new("cargo");
    command.args(["metadata", "--format-version", "1"]);
    // Without it cargo resolves every platform, and fetches crates no build
    // of this machine needs.
    if let Some(host) = host_triple() {
        command.args(["--filter-platform", &host]);
    }
    let output = command.output().map_err(|e| {
        anyhow!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[e.to_string()]
        ))
    })?;
    if !output.status.success() {
        let reason = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[reason]
        ));
    }
    pk_of(&String::from_utf8_lossy(&output.stdout))
}

/// The platform this machine builds for, as `rustc` names it.
fn host_triple() -> Option<String> {
    let output = Command::new("rustc").arg("-vV").output().ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(str::to_string)
}

/// The type the features of `runique` give, in cargo's description of the
/// project (`cargo metadata`): `pk-uuid`, else `big-pk`, else `i32`, the
/// order the framework itself follows.
fn pk_of(metadata: &str) -> Result<PkType> {
    let missing = || {
        anyhow!(t(runique_i18n::keys::makemigrations::error::PROJECT_PK_NO_RUNIQUE).into_owned())
    };
    let json: serde_json::Value = serde_json::from_str(metadata).map_err(|e| {
        anyhow!(tf(
            runique_i18n::keys::makemigrations::error::PROJECT_PK_CARGO,
            &[e.to_string()]
        ))
    })?;
    let id = json["packages"]
        .as_array()
        .and_then(|packages| {
            packages
                .iter()
                .find(|package| package["name"].as_str() == Some("runique"))
        })
        .and_then(|package| package["id"].as_str())
        .ok_or_else(missing)?;
    let features = json["resolve"]["nodes"]
        .as_array()
        .and_then(|nodes| nodes.iter().find(|node| node["id"].as_str() == Some(id)))
        .and_then(|node| node["features"].as_array())
        .ok_or_else(missing)?;
    let has = |name: &str| {
        features
            .iter()
            .any(|feature| feature.as_str() == Some(name))
    };
    Ok(if has("pk-uuid") {
        PkType::Uuid
    } else if has("big-pk") {
        PkType::I64
    } else {
        PkType::I32
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::tests::TempDir;

    /// Cargo's description of a project whose `runique` has `features`.
    fn metadata(features: &[&str]) -> String {
        serde_json::json!({
            "packages": [
                { "name": "serde", "id": "registry+https://example#serde@1.0.0" },
                { "name": "runique", "id": "path+file:///work/runique#4.1.0" },
                { "name": "runique_dsl", "id": "path+file:///work/runique_dsl#0.2.0" },
            ],
            "resolve": { "nodes": [
                { "id": "registry+https://example#serde@1.0.0", "features": ["big-pk", "pk-uuid"] },
                { "id": "path+file:///work/runique_dsl#0.2.0", "features": ["pk-uuid"] },
                { "id": "path+file:///work/runique#4.1.0", "features": features },
            ] },
        })
        .to_string()
    }

    // Only the features of `runique` itself count, not those of a neighbour.
    #[test]
    fn the_type_follows_the_features_of_runique() {
        assert_eq!(
            pk_of(&metadata(&["default", "postgres"])).unwrap(),
            PkType::I32
        );
        assert_eq!(pk_of(&metadata(&[])).unwrap(), PkType::I32);
        assert_eq!(
            pk_of(&metadata(&["postgres", "big-pk"])).unwrap(),
            PkType::I64
        );
        assert_eq!(
            pk_of(&metadata(&["pk-uuid", "admin"])).unwrap(),
            PkType::Uuid
        );
        // The framework's own order when both are on.
        assert_eq!(
            pk_of(&metadata(&["big-pk", "pk-uuid"])).unwrap(),
            PkType::Uuid
        );
        // A name that only looks like one.
        assert_eq!(
            pk_of(&metadata(&["big-pk-extra", "not-pk-uuid"])).unwrap(),
            PkType::I32
        );
    }

    // No `runique` in the project, or a description cargo didn't write:
    // an error, never a type guessed.
    #[test]
    fn a_project_without_runique_is_an_error() {
        let without = serde_json::json!({
            "packages": [{ "name": "serde", "id": "a" }],
            "resolve": { "nodes": [{ "id": "a", "features": [] }] },
        })
        .to_string();
        assert_eq!(
            pk_of(&without).unwrap_err().to_string(),
            t(runique_i18n::keys::makemigrations::error::PROJECT_PK_NO_RUNIQUE)
        );

        // Listed, but absent from what cargo resolved.
        let unresolved = serde_json::json!({
            "packages": [{ "name": "runique", "id": "r" }],
            "resolve": { "nodes": [{ "id": "other", "features": ["big-pk"] }] },
        })
        .to_string();
        assert!(pk_of(&unresolved).is_err());

        assert!(pk_of("not json").is_err());
        assert!(pk_of("{}").is_err());
    }

    #[test]
    fn the_word_is_found_whole_only() {
        assert!(has_word("pk: id => Pk,", "Pk"));
        assert!(has_word("cour_id: Pk [required]", "Pk"));
        assert!(has_word("Pk", "Pk"));
        assert!(!has_word("pk: id => i32", "Pk"));
        assert!(!has_word("PkType Pkg my_Pk Pk2", "Pk"));
        assert!(!has_word("", "Pk"));
    }

    // Only the `.rs` files of the folder are looked at.
    #[test]
    fn the_alias_is_looked_for_in_the_entity_files() {
        let dir = TempDir::new();
        assert!(!alias_used(dir.path()), "empty folder");
        assert!(!alias_used(&dir.join("missing")), "no folder");

        fs::write(dir.join("notes.txt"), "pk: id => Pk").unwrap();
        fs::write(dir.join("shop.rs"), "pk: id => i32").unwrap();
        assert!(!alias_used(dir.path()));

        fs::write(dir.join("book.rs"), "pk: id => Pk").unwrap();
        assert!(alias_used(dir.path()));
    }

    // No model writes `Pk`: the models are read as they are, and cargo is
    // not asked (the folder isn't even a project).
    #[test]
    fn a_project_that_spells_its_types_out_never_asks_cargo() {
        let dir = TempDir::new();
        fs::write(dir.join("shop.rs"), "pk: id => i32").unwrap();
        let read = with_project_pk(dir.path().to_str().unwrap(), || Ok(7)).unwrap();
        assert_eq!(read, 7);
    }
}
