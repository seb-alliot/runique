//! The `runique` command — `new`, `create-superuser`, `test`, `makemigrations`,
//! `migration up|down|status` — read with clap. Only calls: what each command
//! does lives in this crate's library.
use anyhow::Result;
use clap::{Parser, Subcommand};
use runique_cli::{
    create_superuser::create_superuser,
    migration::{makemigrations::makemigrations, steps::apply::migrate},
    new_project::create_new_project,
    test_runner::run_tests,
};
use runique_i18n::Lang;

#[derive(Parser)]
#[command(name = "runique", display_name = "runique-cli", version)]
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
    /// Apply, roll back or list the migrations of the `migration` crate
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
    /// Apply every pending migration
    Up {
        #[arg(long, default_value = "src/entities")]
        entities: String,
        #[arg(long, default_value = "migration/src")]
        migrations: String,
    },
    /// Roll back the last applied migrations
    Down {
        /// How many migrations to roll back
        #[arg(short, long, default_value_t = 1)]
        n: u32,
        #[arg(long, default_value = "migration/src")]
        migrations: String,
    },
    /// List the migrations and whether each one is applied
    Status {
        #[arg(long, default_value = "migration/src")]
        migrations: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    // The project's own translations first: `LANG=nl` names a language only
    // once its folder is read.
    if let Err(error) = runique_i18n::load_project(&runique_i18n::project_dir()) {
        anyhow::bail!(
            "{}",
            runique_i18n::tf(runique_i18n::keys::build::error::LOCALES_INVALID, &[error])
        );
    }
    if let Some(lang) = Lang::from_dotenv().or_else(Lang::from_env) {
        runique_i18n::set_lang(lang);
    }

    let cli = Cli::parse();

    match cli.command {
        Commands::New { name } => create_new_project(&name)?,
        Commands::CreateSuperuser => create_superuser().await?,
        Commands::Test { file, test } => run_tests(file.as_deref(), test.as_deref())?,
        Commands::Migration { action } => match action {
            MigrateAction::Up {
                entities,
                migrations,
            } => migrate::up(&entities, &migrations).await?,
            MigrateAction::Down { n, migrations } => migrate::down(&migrations, n).await?,
            MigrateAction::Status { migrations } => migrate::status(&migrations).await?,
        },
        Commands::Makemigrations {
            entities,
            migrations,
            force,
        } => {
            makemigrations(&entities, &migrations, force).await?;
        }
    }

    Ok(())
}
