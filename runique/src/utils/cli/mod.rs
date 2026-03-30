use crate::utils::trad::{t, tf};
use anyhow::Result;
use rand::RngExt;
use std::{fmt::Write, fs, path::Path};

pub fn create_new_project(name: &str) -> Result<()> {
    validate_project_name(name)?;

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("{}", tf("cli.folder_exists", &[&name]));
    }

    println!("🦀 {}", tf("cli.creating_project", &[&name]));
    let runique_version = env!("CARGO_PKG_VERSION");

    // Créer tous les dossiers nécessaires
    create_project_dirs(project_dir)?;

    // Copier fichiers statiques et templates
    write_project_files(project_dir, name, runique_version)?;

    println!("  {}", tf("cli.project_created", &[&name]));
    println!("\n  {}", t("cli.getting_started"));
    println!("  {}", tf("cli.cd_hint", &[&name]));
    println!("  {}", t("cli.cargo_run_hint"));

    Ok(())
}

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
    Ok(())
}

fn create_project_dirs(project_dir: &Path) -> Result<()> {
    let dirs = [
        "src/entities",
        "src/formulaire",
        "static/css/inscription",
        "static/js",
        "media/favicon",
        "templates/about",
    ];
    for d in dirs {
        fs::create_dir_all(project_dir.join(d))?;
    }
    Ok(())
}

fn write_project_files(project_dir: &Path, name: &str, version: &str) -> Result<()> {
    // Mapping fichiers -> destination
    let files: &[(&[u8], &str)] = &[
        (
            include_bytes!("../../composant-bin/code/views.rs"),
            "src/views.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/forms.rs"),
            "src/formulaire/register.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/login_form.rs"),
            "src/formulaire/login.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/formulaire_mod.rs"),
            "src/formulaire/mod.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/url.rs"),
            "src/url.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/main.rs"),
            "src/main.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/mod.rs"),
            "src/entities/mod.rs",
        ),
        (
            include_bytes!("../../composant-bin/code/users.rs"),
            "src/entities/users.rs",
        ),
        (
            include_bytes!("../../composant-bin/template/index.html"),
            "templates/index.html",
        ),
        (
            include_bytes!("../../composant-bin/template/about.html"),
            "templates/about/about.html",
        ),
        (
            include_bytes!("../../composant-bin/template/inscription_form.html"),
            "templates/inscription_form.html",
        ),
        (
            include_bytes!("../../composant-bin/css/main.css"),
            "static/css/main.css",
        ),
        (
            include_bytes!("../../composant-bin/css/about.css"),
            "static/css/about.css",
        ),
        (
            include_bytes!("../../composant-bin/css/variables.css"),
            "static/css/variables.css",
        ),
        (
            include_bytes!("../../composant-bin/css/inscription/inscription.css"),
            "static/css/inscription/inscription.css",
        ),
        (
            include_bytes!("../../composant-bin/css/inscription/inscription-label.css"),
            "static/css/inscription/inscription-label.css",
        ),
        (
            include_bytes!("../../composant-bin/image/toshiro.avif"),
            "media/toshiro.avif",
        ),
        (
            include_bytes!("../../composant-bin/image/favicon.ico"),
            "media/favicon/favicon.ico",
        ),
        (
            include_bytes!("../../composant-bin/readme/README.md"),
            "README.md",
        ),
        (
            include_bytes!("../../composant-bin/readme/README.fr.md"),
            "README.fr.md",
        ),
        (
            include_bytes!("../../composant-bin/config/ignore"),
            ".gitignore",
        ),
    ];

    for (content, path) in files {
        fs::write(project_dir.join(path), content)?;
    }

    // Cargo.toml et .env avec substitutions
    let cargo_toml = include_str!("../../composant-bin/config/apiconfig")
        .replace("{{PROJECT_NAME}}", name)
        .replace("{{RUNIQUE_VERSION}}", version);
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    let secret_key: String = (0..32).map(|_| rand::rng().random::<u8>()).fold(
        String::with_capacity(64),
        |mut acc, b| {
            write!(&mut acc, "{b:02x}").unwrap();
            acc
        },
    );
    let env_file = include_str!("../../composant-bin/config/secret")
        .replace("your_secret_key_here", &secret_key);
    fs::write(project_dir.join(".env"), env_file)?;

    Ok(())
}
