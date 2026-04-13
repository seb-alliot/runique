//! CLI binary `runique` — `new`, `start`, `makemigration`, `migrate` commands via clap.
use anyhow::Result;
use clap::{Parser, Subcommand};
use runique::utils::{
    cli::{create_superuser, makemigration, migrate, runique_start},
    create_new_project, init_logging,
    trad::{Lang, set_lang},
};

#[derive(Parser)]
#[command(name = "runique")]
#[command(about = "Runique framework CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Runique project
    ///
    /// Generates the complete structure with routes, views, models and templates.
    New { name: String },

    /// Start Runique services (from the project root)
    ///
    /// Automatically detects the project configuration:
    ///   - If `.with_admin(...)` is present in src/main.rs → starts the `AdminPanel` daemon
    ///   - Otherwise → nothing to do
    Start {
        /// Path to src/main.rs (default: ./src/main.rs)
        #[arg(long, default_value = "src/main.rs")]
        main: String,

        /// Path to src/admin.rs (default: ./src/admin.rs)
        #[arg(long, default_value = "src/admin.rs")]
        admin: String,
    },
    /// Create an admin superuser
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
        Commands::CreateSuperuser => create_superuser().await?,
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
