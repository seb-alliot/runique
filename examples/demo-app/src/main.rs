use rusti::{RustiApp, Settings};

mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {


    // Configuration de l'application
    // Vous pouvez personnaliser les settings ici
    // La clef secrète doit être changée pour la production( secret_key dans le server)
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_me_please")
        .build();

    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(url::urls())
        .with_static_files()?
        .with_flash_messages()
        .with_csrf_tokens()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
