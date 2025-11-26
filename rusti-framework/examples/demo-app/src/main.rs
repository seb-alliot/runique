use rusti::{Router, RustiApp, Settings};
use rusti::axum::routing::get;


mod views;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialiser le logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    println!("Starting demo app...");

    // Configuration de l'application
    let settings = Settings::builder()
        .debug(true)
        .templates_dir(vec!["templates".to_string()])
        .static_url("/static")
        .media_url("/media")
        .server("127.0.0.1", 3000)
        .build();

    // Définir les routes
    let routes = Router::new()
        .route("/", get(views::index))
        .route("/about", get(views::about));

    // Créer et lancer l'application
    RustiApp::new(settings).await?
        .routes(routes)
        .with_static_files()?
        .with_sessions()
        .with_default_middleware()
        .run()
        .await?;


    Ok(())
}
