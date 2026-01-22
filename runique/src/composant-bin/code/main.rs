use runique::prelude::*;
mod forms;
mod models;
mod url;
mod views;
use std::env;
// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Logging initialization
    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "sqlx=debug,sea_orm=debug,demo_app=debug".into())
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    // Database connection
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Application configuration !!
    // You can customize the settings parameters here
    // The secret key must be changed for production (secret_key in the server)
    // It can be imported from .env like any environment variable
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    settings.validate_allowed_hosts();

    // Create and run the application
    RuniqueApp::new(settings)
        .await?
        .routes(url::routes())
        .with_database(db)
        .with_static_files()?
        .with_allowed_hosts(
            env::var("ALLOWED_HOSTS")
                .ok()
                .map(|s| s.split(',').map(|h| h.to_string()).collect()),
        )
        .with_sanitize_text_inputs(true)
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}