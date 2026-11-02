//! Hands `runique new` the exact SeaORM requirements runique is built with, read
//! from the manifests: the generated project and its `migration/` crate then
//! pin the same versions, with no copy in the code to keep in sync.
use std::path::Path;

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").expect("set by cargo");
    let manifest = Path::new(&dir).join("Cargo.toml");
    println!("cargo:rerun-if-changed={}", manifest.display());

    let package = read(&manifest);
    for (dep, var) in [
        ("sea-orm", "RUNIQUE_SEA_ORM_REQ"),
        ("sea-orm-migration", "RUNIQUE_SEA_ORM_MIGRATION_REQ"),
    ] {
        let req = requirement(&package, dep, &manifest);
        println!("cargo:rustc-env={var}={req}");
    }
}

fn read(path: &Path) -> toml::Table {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
        .parse()
        .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()))
}

/// The version requirement of `dep`. In the repository it is inherited
/// (`workspace = true`) from the workspace manifest one level up; in the
/// published package cargo has already written it in place.
fn requirement(package: &toml::Table, dep: &str, manifest: &Path) -> String {
    let entry = package
        .get("dependencies")
        .and_then(|d| d.get(dep))
        .unwrap_or_else(|| panic!("{dep} is not a dependency in {}", manifest.display()));
    let inherited = entry.get("workspace").and_then(toml::Value::as_bool) == Some(true);
    let entry = if inherited {
        let root = manifest
            .parent()
            .and_then(Path::parent)
            .expect("the workspace manifest sits above the package")
            .join("Cargo.toml");
        println!("cargo:rerun-if-changed={}", root.display());
        read(&root)
            .get("workspace")
            .and_then(|w| w.get("dependencies"))
            .and_then(|d| d.get(dep))
            .cloned()
            .unwrap_or_else(|| panic!("{dep} missing from [workspace.dependencies]"))
    } else {
        entry.clone()
    };
    entry
        .as_str()
        .or_else(|| entry.get("version").and_then(toml::Value::as_str))
        .unwrap_or_else(|| panic!("{dep} has no version requirement"))
        .to_string()
}
