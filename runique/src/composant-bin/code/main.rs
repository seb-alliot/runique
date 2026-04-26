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

    builder::new(config)
        .routes(url::routes())
        .with_database(db)
        .statics()
        .middleware(|m| {
            m.with_session_memory_limit(5 * 1024 * 1024, 10 * 1024 * 1024)
                .with_session_cleanup_interval(5)
                .with_allowed_hosts(|h| {
                    h.enabled(!is_debug())
                        .host("localhost:3000")
                        .host("127.0.0.1:3000")
                })
        })
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
