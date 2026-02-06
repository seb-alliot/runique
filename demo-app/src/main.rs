#[macro_use]
extern crate runique;

mod forms;
mod models;
mod prelude;
mod url;
mod views;

use prelude::*;
use runique::app::builder::RuniqueAppBuilder as builder;

mod form_test;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Configuration de l'application
    let config = RuniqueConfig::from_env();

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // ═══════════════════════════════════════════════════
    // Builder Intelligent — ordre libre, exécution stricte
    //
    // Peu importe l'ordre d'appel :
    //   .routes() → .with_database() → .statics()
    // Le framework valide tout, puis réorganise
    // les middlewares automatiquement par slots.
    // ═══════════════════════════════════════════════════
    builder::new(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
