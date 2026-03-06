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
use dhat;

use runique::app::builder::RuniqueAppBuilder as builder;

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();


    password_init(PasswordConfig::Manual(Manual::Argon2));

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
