use runique::prelude::*;
mod forms;
mod models;
mod url;
mod views;

use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connexion à la base de données
    let db_config = DatabaseConfig::from_env()?.build();
    let db = db_config.connect().await?;

    let settings = Settings::builder()
        .debug(true)
        .static_url("/static/")
        .staticfiles_dirs("static".to_string())
        .media_root("media".to_string())
        .templates_dir(vec!["templates".to_string()])
        .server("127.0.0.1", 3000, "change_your_secret_key")
        .build();

    settings.validate_allowed_hosts();

    // Créer et lancer l'application
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
        .with_sanitize_text_inputs(false)
        .with_security_headers(CspConfig::strict())
        .with_default_middleware()
        .run()
        .await?;

    Ok(())
}
