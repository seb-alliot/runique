#[macro_use]
extern crate runique;

use runique::prelude::*;
mod forms;
mod models;
mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration de l'application
    let config = RuniqueConfig::from_env();

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Créer et lancer l'application
    RuniqueApp::builder(config)
        .routes(url::routes())
        .with_database(db)
        .build()
        .await?
        .run()
        .await?;

    Ok(())
}
