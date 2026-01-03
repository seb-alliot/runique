use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "runique")]
#[command(about = "runique web framework CLI", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Runique application in the current directory
    NewApp,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::NewApp => {
            if let Err(e) = create_new_app() {
                eprintln!("❌ Erreur: {}", e);
                std::process::exit(1);
            }
        }
    }
}

fn create_new_app() -> io::Result<()> {
    println!("🦀 Initialisation d'une nouvelle application Runique...\n");

    // Vérifier que Cargo.toml existe
    let cargo_toml_path = PathBuf::from("Cargo.toml");
    if !cargo_toml_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Cargo.toml introuvable. Veuillez d'abord exécuter 'cargo init' dans ce dossier.",
        ));
    }

    // Récupérer le chemin du template
    let template_path = get_template_path()?;

    // Fusionner Cargo.toml
    println!("📦 Mise à jour de Cargo.toml...");
    merge_cargo_toml(&cargo_toml_path, &template_path)?;

    // Copier .env
    println!("⚙️  Création du fichier .env...");
    copy_file_with_confirm(&template_path.join(".env"), &PathBuf::from(".env"))?;

    // Créer la structure de dossiers
    println!("📁 Création de la structure de dossiers...");
    create_directory_structure()?;

    // Copier les fichiers sources
    println!("📝 Copie des fichiers sources...");
    copy_source_files(&template_path)?;

    println!("\n Application Runique créée avec succès!");
    println!("\n Prochaines étapes:");
    println!("   1. Configurez votre base de données dans .env");
    println!("   2. Lancez l'application: cargo run");
    println!("   3. Consultez la documentation: https://docs.runique.rs\n");
    Ok(())
}

fn get_template_path() -> io::Result<PathBuf> {
    // Chercher le template dans le dossier d'installation
    let cargo_manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let template_path = cargo_manifest.join("examples").join("new-app");

    if template_path.exists() {
        return Ok(template_path);
    }

    // Alternative: chercher depuis le binaire installé
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let template_path = parent
                .join("..")
                .join("share")
                .join("Runique")
                .join("new-app");
            if template_path.exists() {
                return Ok(template_path);
            }
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Template 'new-app' introuvable. Assurez-vous que Runique est correctement installé.",
    ))
}

fn merge_cargo_toml(existing_path: &Path, template_path: &Path) -> io::Result<()> {
    // Lire le Cargo.toml existant
    let existing_content = fs::read_to_string(existing_path)?;

    // Lire le Cargo.toml du template
    let template_cargo = template_path.join("Cargo.toml");
    let template_content = fs::read_to_string(template_cargo)?;

    // Parser pour extraire les dépendances du template
    let dependencies_section = extract_dependencies_section(&template_content);

    // Vérifier si [dependencies] existe déjà
    let merged_content = if existing_content.contains("[dependencies]") {
        // Remplacer ou fusionner la section dependencies
        replace_dependencies_section(&existing_content, &dependencies_section)
    } else {
        // Ajouter la section dependencies à la fin
        format!("{}\n{}", existing_content.trim(), dependencies_section)
    };

    // Écrire le nouveau Cargo.toml
    fs::write(existing_path, merged_content)?;

    Ok(())
}

fn extract_dependencies_section(content: &str) -> String {
    let mut in_dependencies = false;
    let mut dependencies_lines = Vec::new();

    for line in content.lines() {
        if line.trim().starts_with("[dependencies]") {
            in_dependencies = true;
            dependencies_lines.push(line.to_string());
        } else if in_dependencies {
            if line.trim().starts_with('[') && !line.trim().starts_with("[dependencies") {
                // Nouvelle section, on arrête
                break;
            }
            dependencies_lines.push(line.to_string());
        }
    }

    dependencies_lines.join("\n")
}

fn replace_dependencies_section(existing: &str, new_deps: &str) -> String {
    let mut result = Vec::new();
    let mut in_dependencies = false;
    let mut dependencies_replaced = false;

    for line in existing.lines() {
        if line.trim().starts_with("[dependencies]") {
            if !dependencies_replaced {
                // Ajouter la nouvelle section dependencies
                result.push(new_deps.to_string());
                dependencies_replaced = true;
            }
            in_dependencies = true;
        } else if in_dependencies && line.trim().starts_with('[') {
            // Nouvelle section, on arrête d'ignorer
            in_dependencies = false;
            result.push(line.to_string());
        } else if !in_dependencies {
            result.push(line.to_string());
        }
        // On ignore les lignes dans l'ancienne section dependencies
    }

    result.join("\n")
}

fn create_directory_structure() -> io::Result<()> {
    let directories = vec![
        "src/models",
        "src/static/css",
        "src/static/js",
        "src/media",
        "templates",
        "migration",
    ];

    for dir in directories {
        fs::create_dir_all(dir)?;
    }

    Ok(())
}

fn copy_source_files(template_path: &Path) -> io::Result<()> {
    // Fichiers à copier
    let files_to_copy = vec![
        ("src/main.rs", "src/main.rs"),
        ("src/url.rs", "src/url.rs"),
        ("src/views.rs", "src/views.rs"),
        ("src/forms.rs", "src/forms.rs"),
        ("src/models/mod.rs", "src/models/mod.rs"),
        ("src/models/users.rs", "src/models/users.rs"),
    ];

    for (src, dst) in files_to_copy {
        let src_path = template_path.join(src);
        let dst_path = PathBuf::from(dst);
        copy_file_with_confirm(&src_path, &dst_path)?;
    }

    Ok(())
}

fn copy_file_with_confirm(src: &Path, dst: &Path) -> io::Result<()> {
    // Si le fichier de destination existe, demander confirmation
    if dst.exists() {
        print!(
            " Le fichier '{}' existe déjà. Écraser? [y/N]: ",
            dst.display()
        );
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut response = String::new();
        stdin.lock().read_line(&mut response)?;

        if !response.trim().eq_ignore_ascii_case("y") {
            println!("Continue");
            return Ok(());
        }
    }

    // Créer le dossier parent si nécessaire
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)?;
    }

    // Copier le fichier
    fs::copy(src, dst)?;
    println!("   ✓ {}", dst.display());

    Ok(())
}
