use rusti::{
    RustiApp,
    Settings,
    DatabaseConfig,
};

mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;
    println!("Connected to the database successfully. {}", db_config.engine.name());

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
        .with_database(db)
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
