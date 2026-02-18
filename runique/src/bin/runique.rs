use anyhow::Result;
use clap::{Parser, Subcommand};
use runique::migration::{makemigrations, migrate};
use runique::utils::init_logging;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "runique")]
#[command(about = "CLI du framework Runique", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Créer un nouveau projet Runique
    ///
    /// Génère la structure complète avec routes, vues, modèles et templates.
    New { name: String },

    /// Démarrer les services Runique (depuis la racine du projet)
    ///
    /// Détecte automatiquement la configuration du projet :
    ///   - Si `.with_admin(...)` est présent dans src/main.rs → démarre le daemon AdminPanel
    ///   - Sinon → rien à faire
    Start {
        /// Chemin vers src/main.rs (défaut: ./src/main.rs)
        #[arg(long, default_value = "src/main.rs")]
        main: String,

        /// Chemin vers src/admin.rs (défaut: ./src/admin.rs)
        #[arg(long, default_value = "src/admin.rs")]
        admin: String,
    },
    /// Créer un superuser admin
    CreateSuperuser,
    Migration {
        #[command(subcommand)]
        action: MigrateAction,
    },
    Makemigrations {
        #[arg(long, default_value = "src/entities")]
        entities: String,
        #[arg(long, default_value = "migration/src")]
        migrations: String,
        #[arg(long, default_value = "false")]
        force: bool,
    },
}

#[derive(Subcommand)]
enum MigrateAction {
    Up {
        #[arg(long, default_value = "migration/src")]
        migrations: String,
    },
    Down {
        #[arg(long, default_value = "migration/src")]
        migrations: String,
        #[arg(long, num_args = 1..)]
        files: Vec<String>,
        #[arg(long)]
        batch: Option<String>,
    },
    Status {
        #[arg(long, default_value = "migration/src")]
        migrations: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
        Commands::Start { main, admin } => runique_start(&main, &admin)?,
        Commands::CreateSuperuser => runique::admin::create_superuser().await?,
        Commands::Migration { action } => match action {
            MigrateAction::Up { migrations } => {
                migrate::up(&migrations).await?;
            }
            MigrateAction::Down {
                migrations,
                files,
                batch,
            } => {
                migrate::down(&migrations, files, batch).await?;
            }
            MigrateAction::Status { migrations } => {
                migrate::status(&migrations).await?;
            }
        },
        Commands::Makemigrations {
            entities,
            migrations,
            force,
        } => {
            makemigrations::run(&entities, &migrations, force).await?;
        }
    }

    Ok(())
}

fn runique_start(main_path: &str, admin_path: &str) -> Result<()> {
    let main_file = Path::new(main_path);

    if !main_file.exists() {
        anyhow::bail!(
            "Fichier non trouvé: {}\nAssurez-vous d'être à la racine de votre projet Runique.",
            main_path
        );
    }

    let main_source = fs::read_to_string(main_file)?;

    if !has_admin(&main_source) {
        println!("  Add .with_admin(...) in your builder to enable the AdminPanel.");
        return Ok(());
    }

    println!("Admin detected → starting the daemon");
    // Lancer le daemon en thread séparé
    let admin_path = admin_path.to_string();
    std::thread::spawn(move || {
        if let Err(e) = start_admin_daemon(&admin_path) {
            eprintln!("[Daemon] Erreur: {}", e);
        }
    });

    // Lancer le serveur applicatif après le daemon
    use std::process::Command;
    let status = Command::new("cargo")
        .arg("run")
        .status()
        .expect("Échec du lancement de cargo run");
    if !status.success() {
        anyhow::bail!("Le serveur applicatif n'a pas démarré correctement (cargo run)");
    }
    Ok(())
}

// Détection de .with_admin() dans main.rs

/// Vérifie si `src/main.rs` contient un appel à `.with_admin(...)`
fn has_admin(source: &str) -> bool {
    source.contains(".with_admin(")
}

// Daemon AdminPanel

fn start_admin_daemon(admin_path: &str) -> Result<()> {
    use runique::admin::daemon::watch;

    let admin_file = Path::new(admin_path);

    if !admin_file.exists() {
        anyhow::bail!(
            "Fichier admin non trouvé: {}\nCréez src/admin.rs avec le macro admin!{{}}.",
            admin_path
        );
    }

    watch(admin_file).map_err(|e| anyhow::anyhow!("Erreur daemon: {}", e))?;

    Ok(())
}

// runique new — Création de projet
fn create_new_project(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("The project name cannot be empty");
    }
    if !name
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        anyhow::bail!("The project name must contain only letters, numbers, _ or -");
    }
    if name.starts_with('-') {
        anyhow::bail!("The project name cannot start with -");
    }

    let project_dir = Path::new(name);
    if project_dir.exists() {
        anyhow::bail!("The folder '{}' already exists", name);
    }

    println!("🦀 Creating project '{}'...", name);

    let runique_version = env!("CARGO_PKG_VERSION");

    let view_rs_content = include_bytes!("../composant-bin/code/views.rs").to_vec();
    let formulaire = include_bytes!("../composant-bin/code/forms.rs").to_vec();
    let url_rs = include_bytes!("../composant-bin/code/url.rs").to_vec();
    let main_rs = include_bytes!("../composant-bin/code/main.rs").to_vec();
    let user_exemple = include_bytes!("../composant-bin/code/users.rs").to_vec();
    let mod_rs_content = include_bytes!("../composant-bin/code/mod.rs").to_vec();

    let index_html = include_bytes!("../composant-bin/template/index.html").to_vec();
    let about_html = include_bytes!("../composant-bin/template/about.html").to_vec();
    let inscription_html =
        include_bytes!("../composant-bin/template/inscription_form.html").to_vec();

    let main_css = include_bytes!("../composant-bin/css/main.css").to_vec();
    let about_css = include_bytes!("../composant-bin/css/about.css").to_vec();
    let variable_css = include_bytes!("../composant-bin/css/variables.css").to_vec();
    let inscription_css =
        include_bytes!("../composant-bin/css/inscription/inscription.css").to_vec();
    let inscription_label_css =
        include_bytes!("../composant-bin/css/inscription/inscription-label.css").to_vec();

    let image = include_bytes!("../composant-bin/image/toshiro.avif").to_vec();
    let favicon = include_bytes!("../composant-bin/image/favicon.ico").to_vec();

    let cargo_toml = include_str!("../composant-bin/config/apiconfig")
        .replace("{{PROJECT_NAME}}", name)
        .replace("{{RUNIQUE_VERSION}}", runique_version)
        .into_bytes();

    let env_file = include_bytes!("../composant-bin/config/secret").to_vec();
    let gitignore = include_bytes!("../composant-bin/config/ignore").to_vec();
    let readme_va = include_bytes!("../composant-bin/readme/README.md").to_vec();
    let readme_fr = include_bytes!("../composant-bin/readme/README.fr.md").to_vec();

    fs::create_dir_all(project_dir.join("src/models"))?;
    fs::create_dir_all(project_dir.join("static/css/inscription"))?;
    fs::create_dir_all(project_dir.join("static/js"))?;
    fs::create_dir_all(project_dir.join("media/favicon"))?;
    fs::create_dir_all(project_dir.join("templates/about"))?;

    fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    fs::write(project_dir.join(".env"), env_file)?;
    fs::write(project_dir.join(".gitignore"), gitignore)?;
    fs::write(project_dir.join("README.md"), readme_va)?;
    fs::write(project_dir.join("README.fr.md"), readme_fr)?;

    fs::write(project_dir.join("src/main.rs"), main_rs)?;
    fs::write(project_dir.join("src/forms.rs"), formulaire)?;
    fs::write(project_dir.join("src/url.rs"), url_rs)?;
    fs::write(project_dir.join("src/views.rs"), view_rs_content)?;
    fs::write(project_dir.join("src/models/mod.rs"), mod_rs_content)?;
    fs::write(project_dir.join("src/models/users.rs"), user_exemple)?;

    fs::write(project_dir.join("templates/index.html"), index_html)?;
    fs::write(project_dir.join("templates/about/about.html"), about_html)?;
    fs::write(
        project_dir.join("templates/inscription_form.html"),
        inscription_html,
    )?;

    fs::write(project_dir.join("static/css/main.css"), main_css)?;
    fs::write(project_dir.join("static/css/about.css"), about_css)?;
    fs::write(project_dir.join("static/css/variables.css"), variable_css)?;
    fs::write(
        project_dir.join("static/css/inscription/inscription.css"),
        inscription_css,
    )?;
    fs::write(
        project_dir.join("static/css/inscription/inscription-label.css"),
        inscription_label_css,
    )?;

    fs::write(project_dir.join("media/toshiro.avif"), image)?;
    fs::write(project_dir.join("media/favicon/favicon.ico"), favicon)?;

    println!("  Project '{}' created successfully!", name);
    println!("\n  To get started:");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}
