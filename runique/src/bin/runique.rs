use anyhow::Result;
use clap::{Parser, Subcommand};
use runique::utils::{
    cli::{makemigration, migrate, runique_start},
    create_new_project, init_logging,
    trad::{Lang, set_lang},
};

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
    ///   - Si `.with_admin(...)` est présent dans src/main.rs → démarre le daemon `AdminPanel`
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
    dotenvy::dotenv_override().ok();

    let lang_str = std::env::var("LANG")
        .ok()
        .or_else(|| std::env::var("LC_ALL").ok())
        .or_else(|| std::env::var("LC_MESSAGES").ok());

    if let Some(lang) = lang_str {
        set_lang(Lang::from(lang.as_str()));
    }

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
                migrate::status(&migrations)?;
            }
        },
        Commands::Makemigrations {
            entities,
            migrations,
            force,
        } => {
            makemigration::run(&entities, &migrations, force)?;
        }
    }

    Ok(())
}
