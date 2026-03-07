#[macro_use]
extern crate runique;
mod prelude;
use prelude::*;
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
    password_init(PasswordConfig::Manual(Manual::Argon2));

    let config: RuniqueConfig = RuniqueConfig::from_env();

    let db_config = DatabaseConfig::from_env()?.min_connections(2).build();
    let db: DatabaseConnection = db_config.connect().await?;

    builder::new(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .middleware(|m| {
            m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
                .with_session_cleanup_interval(5)
        })
        .with_admin(|a| {
            a.with_registry(admin::admin_config())
                .hot_reload(cfg!(debug_assertions))
                .site_title("Administration")
                .auth(RuniqueAdminAuth::new())
                .routes(admins::routes("/admin"))
                .with_proto_state(admins::admin_proto_state())
        })
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
