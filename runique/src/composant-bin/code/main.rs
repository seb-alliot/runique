#[macro_use]
extern crate runique;
use runique::prelude::*;
mod entities;
mod formulaire;
mod url;
mod views;

use runique::app::builder::RuniqueAppBuilder as builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    password_init(PasswordConfig::auto_with(Manual::Argon2));

    let config: RuniqueConfig = RuniqueConfig::from_env();

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
