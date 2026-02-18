#[macro_use]
extern crate runique;
use runique::prelude::*;

mod admin;
mod admins;
mod entities;
mod form_test;
mod formulaire;
mod url;
mod views;

use runique::app::builder::RuniqueAppBuilder as builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config: RuniqueConfig = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.build();
    let db: DatabaseConnection = db_config.connect().await?;

    builder::new(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .with_admin(|a| {
            a.with_registry(admin::admin_config())
                .hot_reload(cfg!(debug_assertions))
                .site_title("Administration")
                .auth(RuniqueAdminAuth::new())
                .routes(admins::admin("/admin"))
        })
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
