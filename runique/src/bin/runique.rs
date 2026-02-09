use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "runique")]
#[command(about = "CLI pour créer des projets Runique", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Créer un nouveau projet Runique
    New { name: String },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
    }
    Ok(())
}

fn create_new_project(name: &str) -> Result<()> {
    // === VALIDATION ===
    if name.is_empty() {
        anyhow::bail!("Le nom du projet ne peut pas être vide");
    }

    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("Le nom doit contenir uniquement des lettres, chiffres, _ ou -");
    }

    if name.starts_with('-') {
        anyhow::bail!("Le nom ne peut pas commencer par -");
    }

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("Le dossier '{}' existe déjà", name);
    }

    println!("🦀 Création du projet '{}'...", name);

    // === CONFIGURATION ===
    let runique_version = env!("CARGO_PKG_VERSION");

    // === CHARGEMENT DES TEMPLATES ===

    // Code Rust
    let view_rs_content = include_bytes!("../composant-bin/code/views.rs").to_vec();
    let formulaire = include_bytes!("../composant-bin/code/forms.rs").to_vec();
    let url_rs = include_bytes!("../composant-bin/code/url.rs").to_vec();
    let main_rs = include_bytes!("../composant-bin/code/main.rs").to_vec();
    let user_exemple = include_bytes!("../composant-bin/code/users.rs").to_vec();
    let mod_rs_content = include_bytes!("../composant-bin/code/mod.rs").to_vec();

    // Templates HTML
    let index_html = include_bytes!("../composant-bin/template/index.html").to_vec();
    let about_html = include_bytes!("../composant-bin/template/about.html").to_vec();
    let inscription_html = include_bytes!("../composant-bin/template/inscription_form.html").to_vec();

    // CSS
    let main_css = include_bytes!("../composant-bin/css/main.css").to_vec();
    let about_css = include_bytes!("../composant-bin/css/about.css").to_vec();
    let variable_css = include_bytes!("../composant-bin/css/variables.css").to_vec();
    let inscription_css = include_bytes!("../composant-bin/css/inscription/inscription.css").to_vec();
    let inscription_label_css = include_bytes!("../composant-bin/css/inscription/inscription-label.css").to_vec();

    // Images
    let image = include_bytes!("../composant-bin/image/toshiro.avif").to_vec();
    let favicon = include_bytes!("../composant-bin/image/favicon.ico").to_vec();

    // Fichiers de configuration
    let cargo_toml = include_str!("../composant-bin/config/apiconfig")
        .replace("{{PROJECT_NAME}}", name)
        .replace("{{RUNIQUE_VERSION}}", runique_version)
        .into_bytes();
    
    let env_file = include_bytes!("../composant-bin/config/secret").to_vec();
    let gitignore = include_bytes!("../composant-bin/config/ignore").to_vec();
    let readme_va = include_bytes!("../composant-bin/readme/README.md").to_vec();
    let readme_fr = include_bytes!("../composant-bin/readme/README.fr.md").to_vec();

    // === CRÉATION DES DOSSIERS ===
    fs::create_dir_all(project_dir.join("src/models"))?;
    fs::create_dir_all(project_dir.join("static/css/inscription"))?;
    fs::create_dir_all(project_dir.join("static/js"))?;
    fs::create_dir_all(project_dir.join("media/favicon"))?;
    fs::create_dir_all(project_dir.join("templates/about"))?;

    // === ÉCRITURE DES FICHIERS ===

    // Racine
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(project_dir.join(".env"), env_file)?;
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    fs::write(project_dir.join("README.md"), readme_va)?;
    fs::write(project_dir.join("README.fr.md"), readme_fr)?;

    // Source Rust
    fs::write(project_dir.join("src/main.rs"), main_rs)?;
    fs::write(project_dir.join("src/forms.rs"), formulaire)?;
    fs::write(project_dir.join("src/url.rs"), url_rs)?;
    fs::write(project_dir.join("src/views.rs"), view_rs_content)?;
    fs::write(project_dir.join("src/models/mod.rs"), mod_rs_content)?;
    fs::write(project_dir.join("src/models/users.rs"), user_exemple)?;

    // HTML
    fs::write(project_dir.join("templates/index.html"), index_html)?;
    fs::write(project_dir.join("templates/about/about.html"), about_html)?;
    fs::write(project_dir.join("templates/inscription_form.html"), inscription_html)?;

    // CSS
    fs::write(project_dir.join("static/css/main.css"), main_css)?;
    fs::write(project_dir.join("static/css/about.css"), about_css)?;
    fs::write(project_dir.join("static/css/variables.css"), variable_css)?;
    fs::write(project_dir.join("static/css/inscription/inscription.css"), inscription_css)?;
    fs::write(project_dir.join("static/css/inscription/inscription-label.css"), inscription_label_css)?;

    // Media
    fs::write(project_dir.join("media/toshiro.avif"), image)?;
    fs::write(project_dir.join("media/favicon/favicon.ico"), favicon)?;

    // === MESSAGE DE SUCCÈS ===
    println!(" ✅ Projet '{}' créé avec succès !", name);
    println!("\n  Pour démarrer :");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}