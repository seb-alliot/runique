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
    let runique_version = "1.0.86";

    // === CHARGEMENT DES TEMPLATES ===

    // Code Rust
    let view_rs_content = include_bytes!("../composant-bin/code/views.rs").to_vec();
    let formulaire = include_bytes!("../composant-bin/code/forms.rs").to_vec();
    let user_exemple = include_bytes!("../composant-bin/code/users.rs").to_vec();
    let mod_rs_content = include_bytes!("../composant-bin/code/mod.rs").to_vec();
    let url_rs = include_bytes!("../composant-bin/code/url.rs").to_vec();
    let main_rs = include_bytes!("../composant-bin/code/main.rs").to_vec();

    // Templates HTML
    let index_html = include_bytes!("../composant-bin/template/index.html").to_vec();
    let about_html = include_bytes!("../composant-bin/template/about.html").to_vec();
    let view_user_html = include_bytes!("../composant-bin/template/view_user.html").to_vec();
    let register_user_html = include_bytes!("../composant-bin/template/register_user.html").to_vec();

    // CSS
    let main_css = include_bytes!("../composant-bin/css/main.css").to_vec();
    let about_css = include_bytes!("../composant-bin/css/about.css").to_vec();
    let register_form_css = include_bytes!("../composant-bin/css/register-form.css").to_vec();
    let search_user_css = include_bytes!("../composant-bin/css/search-user.css").to_vec();
    let variable_css = include_bytes!("../composant-bin/css/variables.css").to_vec();

    // Images
    let image = include_bytes!("../composant-bin/image/toshiro.jpg").to_vec();
    let favicon = include_bytes!("../composant-bin/image/favicon.ico").to_vec();
    // Fichiers de configuration
    let cargo_toml = include_str!("../composant-bin/config/apiconfig")
        .replace("{{PROJECT_NAME}}", name)
        .replace("{{RUNIQUE_VERSION}}", runique_version)
        .to_string()
        .into_bytes();
    let env_file = include_bytes!("../composant-bin/config/secret").to_vec();
    let gitignore = include_bytes!("../composant-bin/config/ignore").to_vec();
    let readme_va = include_bytes!("../composant-bin/readme/README.md").to_vec();
    let readme_fr = include_bytes!("../composant-bin/readme/README.fr.md").to_vec();

    // === CRÉATION DES DOSSIERS ===
    fs::create_dir_all(project_dir)?;
    fs::create_dir_all(project_dir.join("src/models"))?;
    fs::create_dir_all(project_dir.join("src/static/css"))?;
    fs::create_dir_all(project_dir.join("src/static/js"))?;
    fs::create_dir_all(project_dir.join("src/static/images"))?;
    fs::create_dir_all(project_dir.join("src/media"))?;
    fs::create_dir_all(project_dir.join("src/media/favicon"))?;
    fs::create_dir_all(project_dir.join("templates"))?;
    fs::create_dir_all(project_dir.join("templates/profile"))?;
    fs::create_dir_all(project_dir.join("templates/about"))?;

    // === ÉCRITURE DES FICHIERS ===

    // Fichiers racine
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(project_dir.join(".env"), env_file)?;
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    fs::write(project_dir.join("README.md"), readme_va)?;
    fs::write(project_dir.join("README.fr.md"), readme_fr)?;

    // Code Rust sources
    fs::write(project_dir.join("src/main.rs"), main_rs)?;
    fs::write(project_dir.join("src/forms.rs"), formulaire)?;
    fs::write(project_dir.join("src/url.rs"), url_rs)?;
    fs::write(project_dir.join("src/views.rs"), view_rs_content)?;
    fs::write(project_dir.join("src/models/mod.rs"), mod_rs_content)?;
    fs::write(project_dir.join("src/models/users.rs"), user_exemple)?;

    // Templates HTML
    fs::write(project_dir.join("templates/index.html"), index_html)?;
    fs::write(project_dir.join("templates/about/about.html"), about_html)?;
    fs::write(
        project_dir.join("templates/profile/view_user.html"),
        view_user_html,
    )?;
    fs::write(
        project_dir.join("templates/profile/register_user.html"),
        register_user_html,
    )?;

    // CSS
    fs::write(project_dir.join("src/static/css/main.css"), main_css)?;
    fs::write(project_dir.join("src/static/css/about.css"), about_css)?;
    fs::write(
        project_dir.join("src/static/css/register-form.css"),
        register_form_css,
    )?;
    fs::write(
        project_dir.join("src/static/css/search-user.css"),
        search_user_css,
    )?;
    fs::write(
        project_dir.join("src/static/css/variables.css"),
        variable_css,
    )?;

    // Images
    fs::write(project_dir.join("src/media/toshiro.jpg"), image)?;
    fs::write(project_dir.join("src/media/favicon/favicon.ico"), favicon)?;

    // === MESSAGE DE SUCCÈS ===
    println!("   Projet '{}' créé avec succès !", name);
    println!("   Structure créée :");
    println!("   {}/", name);
    println!("   ├── src/");
    println!("   │   ├── models/");
    println!("   │   │   ├── mod.rs");
    println!("   │   │   └── users.rs");
    println!("   │   ├── static/");
    println!("   │   │   ├── css/");
    println!("   │   │   |   ├── variables.css");
    println!("   │   │   |   ├── main.css");
    println!("   │   │   |   ├── register-form.css");
    println!("   │   │   |   ├── search-user.css");
    println!("   │   │   |   └── about.css");
    println!("   │   │   ├── js/");
    println!("   │   │   └── images/");
    println!("   │   ├── media/");
    println!("   │   │   ├── favicon/");
    println!("   │   │   │   └── favicon.ico");
    println!("   │   │   └── toshiro.jpg");
    println!("   │   ├── forms.rs");
    println!("   │   ├── main.rs");
    println!("   │   ├── url.rs");
    println!("   │   └── views.rs");
    println!("   ├── templates/");
    println!("   │   ├── about/");
    println!("   │   │   └── about.html");
    println!("   │   ├── profile/");
    println!("   │   │   ├── register_user.html");
    println!("   │   │   └── view_user.html");
    println!("   │   └── index.html");
    println!("   ├── .env");
    println!("   ├── .gitignore");
    println!("   └── Cargo.toml");
    println!("\n     Pour démarrer :");
    println!("   cd {}", name);
    println!("   cargo run");

    Ok(())
}
