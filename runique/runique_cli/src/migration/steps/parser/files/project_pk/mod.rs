//! The type `Pk` stands for in the project the command runs in.
//!
//! `Pk` follows a feature of `runique` (`big-pk`, `pk-uuid`). The command is
//! built once and serves every project: its own features say nothing of the
//! project's, so it asks cargo which ones the project turns on.
mod alias_used;
mod from_cargo;
mod has_word;
mod host_triple;
mod pk_of;
mod with_project_pk;

use anyhow::{Result, anyhow, bail};
use runique_dsl::ast::PkType;
use runique_i18n::{t, tf};
use std::{fs, path::Path, process::Command};

use alias_used::alias_used;
pub(crate) use from_cargo::from_cargo;
use has_word::has_word;
use host_triple::host_triple;
use pk_of::pk_of;
pub(crate) use with_project_pk::with_project_pk;

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
