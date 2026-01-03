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
    println!("🦀 Création du projet '{}'...", name);

    let project_dir = Path::new(name);

    // Créer la structure de dossiers
    fs::create_dir_all(&project_dir)?;
    fs::create_dir_all(project_dir.join("src/models"))?;
    fs::create_dir_all(project_dir.join("src/static/css"))?;
    fs::create_dir_all(project_dir.join("src/static/js"))?;
    fs::create_dir_all(project_dir.join("src/static/images"))?;
    fs::create_dir_all(project_dir.join("src/media"))?;
    fs::create_dir_all(project_dir.join("templates"))?;

    // Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2024"

[dependencies]
runique = "1.0"
serde = "1.0"
"#,
        name
    );
    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;

    // src/main.rs
    let main_rs = r#"use runique::prelude::*;

mod forms;
mod models;
mod url;
mod views;

#[tokio::main]
async fn main() {
    let app = App::new();
    app.run("127.0.0.1:8000").await.unwrap();
}
"#;
    fs::write(project_dir.join("src/main.rs"), main_rs)?;

    // src/forms.rs
    fs::write(project_dir.join("src/forms.rs"), "// Vos formulaires ici\n")?;

    // src/url.rs
    fs::write(project_dir.join("src/url.rs"), "// Vos routes ici\n")?;

    // src/views.rs
    fs::write(project_dir.join("src/views.rs"), "// Vos vues ici\n")?;

    // src/models/mod.rs
    fs::write(
        project_dir.join("src/models/mod.rs"),
        "// Vos modèles ici\n",
    )?;
    // Fichiers statiques
    fs::write(
        project_dir.join("src/static/css"),
        "/* Vos styles CSS ici */\n",
    )?;
    fs::write(project_dir.join("src/static/js"), "// Vos scripts JS ici\n")?;
    fs::write(project_dir.join("src/static/images"), "")?;
    // Dossier media
    fs::write(project_dir.join("src/media"), "")?;
    // .env
    let env_file = r#"DATABASE_URL=sqlite://db.sqlite
DEBUG=true
SECRET_KEY=change-me-in-production
"#;
    fs::write(project_dir.join(".env"), env_file)?;

    // .gitignore
    let gitignore = r#"/target/
*.db
*.sqlite
.env
"#;
    fs::write(project_dir.join(".gitignore"), gitignore)?;

    println!("   Projet '{}' créé avec succès !", name);
    Ok(())
}
