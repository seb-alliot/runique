use rusti::{
    RustiApp,
    Settings,
    DatabaseConfig,
    tokio,
    CspConfig,
};
mod url;
mod views;
mod models;
mod forms;


// use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialisation du logging
    // tracing_subscriber::registry()
    //     .with(
    //         tracing_subscriber::EnvFilter::try_from_default_env()
    //             .unwrap_or_else(|_| "sqlx=debug,sea_orm=debug,demo_app=debug".into())
    //     )
    //     .with(tracing_subscriber::fmt::layer())
    //     .init();

    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    // Configuration de l'application !!
    // Vous pouvez personnaliser les paramètres du settings ici
    // La clef secrète doit être changée pour la production( secret_key dans the server)
    // elle peux être importé du .env comme toute variable d'environnement
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secrete_key")
        .build();


    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(url::urls())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .with_sanitize_text_inputs(false)
        .with_database(db)
        .run()
        .await?;


    Ok(())
}