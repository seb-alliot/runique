//! CLI binary `runique` — `new`, `makemigration`, `migrate` commands via clap.
use anyhow::Result;
use clap::{Parser, Subcommand};
use runique::{
    cli::{create_new_project, create_superuser, makemigration, migrate, run_tests},
    utils::{
        init_logging,
        trad::{Lang, set_lang},
    },
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

    /// Create an admin superuser
    CreateSuperuser,

    /// Run the project's runique tests (src/runique_test/)
    ///
    /// Each test runs against your database inside a transaction that always
    /// gets rolled back. `runique test` runs them all, `runique test user` only
    /// src/runique_test/user.rs, `runique test user add_email` a single test.
    Test {
        /// Test file in src/runique_test/, without the .rs
        file: Option<String>,
        /// Test function in that file
        test: Option<String>,
    },
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let _log_guards = init_logging();
    dotenvy::dotenv_override().ok();

    if let Some(lang) = Lang::from_env() {
        set_lang(lang);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
        Commands::CreateSuperuser => create_superuser().await?,
        Commands::Test { file, test } => run_tests(file.as_deref(), test.as_deref())?,
        Commands::Migration { action } => match action {
            MigrateAction::Up { migrations } => {
                migrate::up(&migrations).await?;
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
