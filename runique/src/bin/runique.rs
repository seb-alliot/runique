use anyhow::Result;
use clap::{Parser, Subcommand};
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

        /// Dossier de sortie du daemon (défaut: target/runique/admin)
        #[arg(long, default_value = "target/runique/admin")]
        output: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
        Commands::Start {
            main,
            admin,
            output,
        } => runique_start(&main, &admin, &output)?,
    }

    Ok(())
}

fn runique_start(main_path: &str, admin_path: &str, output: &str) -> Result<()> {
    let main_file = Path::new(main_path);

    if !main_file.exists() {
        anyhow::bail!(
            "Fichier non trouvé: {}\nAssurez-vous d'être à la racine de votre projet Runique.",
            main_path
        );
    }

    println!("🔍 Analyse de {}...", main_path);

    let main_source = fs::read_to_string(main_file)?;

    if !has_admin(&main_source) {
        println!("  Aucun AdminPanel détecté dans {}", main_path);
        println!("  Ajoutez .with_admin(...) dans votre builder pour activer l'AdminPanel.");
        return Ok(());
    }

    println!("Admin détecté → démarrage du daemon");
    // Lancer le daemon en thread séparé
    let admin_path = admin_path.to_string();
    let output = output.to_string();
    std::thread::spawn(move || {
        if let Err(e) = start_admin_daemon(&admin_path, &output) {
            eprintln!("[Daemon] Erreur: {}", e);
        }
    });

    // Lancer le serveur applicatif après le daemon
    println!("\n🚀 Lancement du serveur applicatif (cargo run)...\n");
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

fn start_admin_daemon(admin_path: &str, output: &str) -> Result<()> {
    use runique::admin::daemon::watch;

    let admin_file = Path::new(admin_path);
    let output_path = Path::new(output);

    if !admin_file.exists() {
        anyhow::bail!(
            "Fichier admin non trouvé: {}\nCréez src/admin.rs avec le macro admin!{{}}.",
            admin_path
        );
    }
    println!("   (Ctrl+C pour arrêter)\n");

    watch(admin_file, output_path).map_err(|e| anyhow::anyhow!("Erreur daemon: {}", e))?;

    Ok(())
}

// runique new — Création de projet

fn create_new_project(name: &str) -> Result<()> {
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

    println!("  Projet '{}' créé avec succès !", name);
    println!("\n  Pour démarrer :");
    println!("  cd {}", name);
    println!("  cargo run");

    Ok(())
}
