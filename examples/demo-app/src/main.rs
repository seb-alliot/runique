use rusti::{RustiApp, Settings};

mod url;
mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Configuration de l'application
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .static_url("/static")
        .media_url("/media")
        .server("127.0.0.1", 3000)
        .build();

    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(url::urls())
        .with_static_files()?
        .with_flash_messages()
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
