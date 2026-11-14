//! `new` command — scaffolds a new Runique project (files, dependencies, SECRET_KEY).
use crate::atomic::{AtomicBuilder, SourceItems};
use crate::migration::cli_migration::init_migrate::{
    self, Engine, SEA_ORM_MIGRATION_REQ, SEA_ORM_REQ,
};
use crate::utils::trad::{t, tf};
use anyhow::Result;
use rand::RngExt;
use std::{fs, path::Path};

/// Scaffolds a new Runique project named `name`: creates the standard directory
/// layout, writes the bundled example files (views, forms, templates, CSS,
/// `Cargo.toml`) and its `migration` crate, and generates a fresh random
/// `SECRET_KEY` into `.env`.
/// Fails if `name` is invalid or a folder with that name already exists.
pub fn create_new_project(name: &str) -> Result<()> {
    validate_project_name(name)?;

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("{}", tf("cli.folder_exists", &[&name]));
    }

    println!("🦀 {}", tf("cli.creating_project", &[&name]));
    let runique_version = env!("CARGO_PKG_VERSION");

    // Every file of the project in one batch: all of them, or nothing at all
    write_project_files(project_dir, name, runique_version)?;

    println!("  {}", tf("cli.project_created", &[&name]));
    println!("\n  {}", t("cli.getting_started"));
    println!("  {}", tf("cli.cd_hint", &[&name]));
    // Before `cargo run`: the framework's tables (accounts, sessions) must
    // exist, or the first request fails on a missing table.
    println!("  {}", t("cli.migration_up_hint"));
    println!("  {}", t("cli.cargo_run_hint"));

    Ok(())
}

/// Names a project can't take, compared lowercase with `-` read as `_`:
/// cargo refuses Rust's keywords and built-in crates, and its own build
/// folders; Windows refuses its device names as folders; and the generated
/// workspace already holds a `migration` crate and depends on `runique`.
const RESERVED_NAMES: &[&str] = &[
    // Rust keywords, reserved ones included
    "abstract",
    "as",
    "async",
    "await",
    "become",
    "box",
    "break",
    "const",
    "continue",
    "crate",
    "do",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "final",
    "fn",
    "for",
    "gen",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "macro",
    "match",
    "mod",
    "move",
    "mut",
    "override",
    "priv",
    "pub",
    "ref",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "try",
    "type",
    "typeof",
    "unsafe",
    "unsized",
    "use",
    "virtual",
    "where",
    "while",
    "yield",
    // Rust's built-in crates
    "alloc",
    "core",
    "proc_macro",
    "std",
    "test",
    // cargo's build folders
    "build",
    "deps",
    "examples",
    "incremental",
    // Windows device names
    "aux",
    "con",
    "nul",
    "prn",
    "com1",
    "com2",
    "com3",
    "com4",
    "com5",
    "com6",
    "com7",
    "com8",
    "com9",
    "lpt1",
    "lpt2",
    "lpt3",
    "lpt4",
    "lpt5",
    "lpt6",
    "lpt7",
    "lpt8",
    "lpt9",
    // The generated workspace
    "migration",
    "runique",
];

fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("{}", t("cli.name_empty"));
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("{}", t("cli.name_invalid"));
    }
    if name.starts_with('-') {
        anyhow::bail!("{}", t("cli.name_dash"));
    }
    if name.starts_with(|c: char| c.is_ascii_digit()) {
        anyhow::bail!("{}", t("cli.name_digit"));
    }
    let normalized = name.to_lowercase().replace('-', "_");
    if RESERVED_NAMES.contains(&normalized.as_str()) {
        anyhow::bail!("{}", tf("cli.name_reserved", &[name]));
    }
    Ok(())
}

fn write_project_files(project_dir: &Path, name: &str, version: &str) -> Result<()> {
    // Files -> destination mapping
    let files: &[(&[u8], &str)] = &[
        (
            include_bytes!("composant-bin/code/views.rs"),
            "src/views.rs",
        ),
        (
            include_bytes!("composant-bin/code/signup_form.rs"),
            "src/forms/signup.rs",
        ),
        (
            include_bytes!("composant-bin/code/login_form.rs"),
            "src/forms/login.rs",
        ),
        (
            include_bytes!("composant-bin/code/forms_mod.rs"),
            "src/forms/mod.rs",
        ),
        (include_bytes!("composant-bin/code/url.rs"), "src/url.rs"),
        (include_bytes!("composant-bin/code/main.rs"), "src/main.rs"),
        (
            include_bytes!("composant-bin/template/base.html"),
            "templates/base.html",
        ),
        (
            include_bytes!("composant-bin/template/index.html"),
            "templates/index.html",
        ),
        (
            include_bytes!("composant-bin/template/about.html"),
            "templates/about.html",
        ),
        (
            include_bytes!("composant-bin/template/signup.html"),
            "templates/signup.html",
        ),
        (
            include_bytes!("composant-bin/template/login.html"),
            "templates/login.html",
        ),
        (
            include_bytes!("composant-bin/css/main.css"),
            "static/css/main.css",
        ),
        (
            include_bytes!("composant-bin/css/about.css"),
            "static/css/about.css",
        ),
        (
            include_bytes!("composant-bin/css/variables.css"),
            "static/css/variables.css",
        ),
        (
            include_bytes!("composant-bin/css/forms/forms.css"),
            "static/css/forms/forms.css",
        ),
        (
            include_bytes!("composant-bin/css/forms/forms-label.css"),
            "static/css/forms/forms-label.css",
        ),
        (
            include_bytes!("composant-bin/image/runique.avif"),
            "media/runique.avif",
        ),
        (
            include_bytes!("composant-bin/image/favicon.ico"),
            "media/favicon/favicon.ico",
        ),
        (include_bytes!("composant-bin/config/ignore"), ".gitignore"),
    ];

    let mut batch = AtomicBuilder::create();
    for (content, path) in files {
        batch = batch.add(
            project_dir.join(path),
            SourceItems::Bytes(content.to_vec()),
            false,
        )?;
    }
    batch = batch.add(
        project_dir.join("src/entities/mod.rs"),
        SourceItems::Bytes(Vec::new()),
        false,
    )?;
    // Cargo.toml, READMEs and .env with substitutions
    for (template, path) in [
        (include_str!("composant-bin/config/apiconfig"), "Cargo.toml"),
        (include_str!("composant-bin/readme/README.md"), "README.md"),
        (
            include_str!("composant-bin/readme/README.fr.md"),
            "README.fr.md",
        ),
    ] {
        let content = template
            .replace("{{PROJECT_NAME}}", name)
            .replace("{{RUNIQUE_VERSION}}", version)
            .replace("{{SEA_ORM_REQ}}", SEA_ORM_REQ)
            .replace("{{SEA_ORM_MIGRATION_REQ}}", SEA_ORM_MIGRATION_REQ);
        batch = batch.add(
            project_dir.join(path),
            SourceItems::Bytes(content.into_bytes()),
            false,
        )?;
    }

    // SQLite, as the `.env` and the `Cargo.toml` above.
    for (path, content) in
        init_migrate::files(&project_dir.join("migration"), Engine::Sqlite, version)
    {
        batch = batch.add(path, SourceItems::Bytes(content), false)?;
    }

    let secret_key: String = generate_secret_key();
    let env_file =
        include_str!("composant-bin/config/secret").replace("your_secret_key_here", &secret_key);
    batch
        .add(
            project_dir.join(".env"),
            SourceItems::Bytes(env_file.into_bytes()),
            true,
        )?
        .start()?;

    // The only folder of the layout that holds no file: made once the project exists.
    fs::create_dir_all(project_dir.join("static/js"))?;
    Ok(())
}

fn generate_secret_key() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Written from cargo-mutants survivors (2026-10-02): the name becomes a
/// directory, so nothing that could leave the current one may pass.
#[cfg(test)]
mod name_tests {
    use super::validate_project_name;

    #[test]
    fn only_plain_names_are_accepted() {
        for ok in ["blog", "my_app", "my-app-2", "Été"] {
            assert!(validate_project_name(ok).is_ok(), "{ok}");
        }
        for bad in ["", "../escape", "a/b", "a b", "a.b", "-flag", "C:\\x"] {
            assert!(validate_project_name(bad).is_err(), "{bad:?}");
        }
    }

    /// Accepted by the character check, refused by cargo, Windows or the
    /// generated workspace: refused here instead, before any folder exists.
    #[test]
    fn reserved_names_are_refused_whatever_their_case_or_dash() {
        for bad in [
            "test",
            "std",
            "Core",
            "proc-macro",
            "proc_macro",
            "fn",
            "self",
            "deps",
            "CON",
            "nul",
            "lpt1",
            "migration",
            "Runique",
            "1app",
        ] {
            assert!(validate_project_name(bad).is_err(), "{bad:?}");
        }
        for ok in [
            "tests",
            "my_test",
            "testing",
            "app1",
            "console",
            "migrations",
        ] {
            assert!(validate_project_name(ok).is_ok(), "{ok:?}");
        }
    }
}

/// The generated project carries its name and runique's version, and no
/// placeholder is left in it (the README once said "v1.1.54" and "{}/").
#[cfg(test)]
mod generated_files_tests {
    use super::{SEA_ORM_MIGRATION_REQ, SEA_ORM_REQ, write_project_files};

    #[test]
    fn no_placeholder_is_left_in_the_generated_files() {
        let dir = std::env::temp_dir().join(format!("rq_new_{}", uuid::Uuid::new_v4()));
        write_project_files(&dir, "shop", "9.8.7").unwrap();
        for file in [
            "Cargo.toml",
            "README.md",
            "README.fr.md",
            ".env",
            "migration/Cargo.toml",
        ] {
            let content = std::fs::read_to_string(dir.join(file)).unwrap();
            assert!(
                !content.contains("{{") && !content.contains("{}/"),
                "{file}"
            );
        }
        let readme = std::fs::read_to_string(dir.join("README.md")).unwrap();
        assert!(
            readme.contains("shop/") && readme.contains("v9.8.7"),
            "name and version"
        );
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(
            cargo.contains(&format!(r#"sea-orm = "{SEA_ORM_REQ}""#)),
            "{cargo}"
        );
        assert!(cargo.contains(r#"members = ["migration"]"#), "{cargo}");
        let migration = std::fs::read_to_string(dir.join("migration/Cargo.toml")).unwrap();
        assert!(
            migration.contains(r#"runique = { version = "9.8.7""#),
            "the migration crate depends on runique"
        );
        assert!(migration.contains(&format!(r#"version = "{SEA_ORM_MIGRATION_REQ}""#)));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(unix)]
    fn the_generated_env_is_readable_by_its_owner_only() {
        use std::os::unix::fs::PermissionsExt;
        let dir = std::env::temp_dir().join(format!("rq_new_{}", uuid::Uuid::new_v4()));
        write_project_files(&dir, "shop", "9.8.7").unwrap();
        let mode = |file: &str| {
            std::fs::metadata(dir.join(file))
                .unwrap()
                .permissions()
                .mode()
                & 0o777
        };
        assert_eq!(mode(".env"), 0o600, "it holds SECRET_KEY");
        assert_ne!(
            mode("Cargo.toml"),
            0o600,
            "an ordinary file is left as it was"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_generated_env_sets_the_mode_and_the_error_pages() {
        let dir = std::env::temp_dir().join(format!("rq_new_env_{}", uuid::Uuid::new_v4()));
        write_project_files(&dir, "shop", "9.8.7").unwrap();
        let env = std::fs::read_to_string(dir.join(".env")).unwrap();
        let _ = std::fs::remove_dir_all(&dir);

        let lines: Vec<&str> = env.lines().map(str::trim).collect();
        assert!(lines.contains(&"RUNIQUE_ENV=dev"), "{env}");
        assert!(lines.contains(&"DEBUG=true"), "{env}");
        let comment_before = |key: &str| {
            let at = lines.iter().position(|l| l.starts_with(key)).unwrap();
            lines[..at]
                .iter()
                .rev()
                .take_while(|l| !l.is_empty())
                .any(|l| l.starts_with('#'))
        };
        assert!(comment_before("RUNIQUE_ENV="), "{env}");
        assert!(comment_before("DEBUG="), "{env}");
        assert!(!env.contains("your_secret_key_here"), "{env}");
    }

    /// The generated project builds against this checkout of runique: the only
    /// test that catches the scaffold falling behind the API. Slow on its first
    /// run (it compiles SeaORM), hence ignored; the CI runs it with
    /// `--ignored`. The target folder is kept between runs.
    #[test]
    #[ignore = "compiles a whole project: run with --ignored"]
    fn the_generated_project_compiles() {
        let runique = env!("CARGO_MANIFEST_DIR");
        let dir = std::env::temp_dir().join(format!("rq_new_build_{}", uuid::Uuid::new_v4()));
        write_project_files(&dir, "generated_app", "0.0.0").unwrap();
        for manifest in ["Cargo.toml", "migration/Cargo.toml"] {
            let path = dir.join(manifest);
            let content = std::fs::read_to_string(&path).unwrap().replace(
                r#"runique = { version = "0.0.0""#,
                // A literal TOML string: a Windows path keeps its backslashes.
                &format!("runique = {{ path = '{runique}'"),
            );
            std::fs::write(&path, content).unwrap();
        }

        let status = std::process::Command::new(env!("CARGO"))
            .args(["check", "--workspace"])
            .current_dir(&dir)
            .env(
                "CARGO_TARGET_DIR",
                std::path::Path::new(runique).join("../target/generated-project"),
            )
            // Read at compile time by runique's macros: the CI exports
            // `DB_ENGINE=postgres`, the generated project is SQLite.
            .env_remove("DB_ENGINE")
            .env_remove("DATABASE_URL")
            .status()
            .expect("cargo runs");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(status.success(), "the generated project does not compile");
    }
}
