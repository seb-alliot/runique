# Exemple d'utilisation du builder

```rust,ignore
use runique::prelude::*;

#[tokio::main]
async fn main() {
    // Configuration chargée depuis les variables d'environnement
    let config = RuniqueConfig::from_env();

    // Définition des routes de l'application
    let router = axum::Router::new()
        .route("/", axum::routing::get(home_handler))
        .route("/about", axum::routing::get(about_handler));

    // Construction de l'application
    let app = RuniqueApp::builder(config)
        .routes(router)
        .middleware(|m| {
            m.with_session_duration(tower_sessions::cookie::time::Duration::hours(2))
             .with_csp(|c| {
                 c.with_header_security(true)
                  .with_nonce(true)
                  .scripts(vec!["'self'"])
                  .images(vec!["'self'", "data:"])
             })
        })
        .build()
        .await
        .expect("Erreur de démarrage");

    app.run().await;
}
```

## Avec base de données

```rust,ignore
use runique::prelude::*;
use runique::db::DatabaseConfig;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();

    // Option 1 : connexion établie avant le build
    let db = DatabaseConfig::from_env()
        .expect("Config DB manquante")
        .build()
        .connect()
        .await
        .expect("Connexion DB échouée");

    let app = RuniqueApp::builder(config)
        .with_database(db)
        .routes(axum::Router::new())
        .build()
        .await
        .expect("Erreur de démarrage");

    app.run().await;
}
```

## Avec AdminPanel

```rust,ignore
use runique::prelude::*;

#[tokio::main]
async fn main() {
    let config = RuniqueConfig::from_env();

    let app = RuniqueApp::builder(config)
        .routes(axum::Router::new())
        .with_admin(|a| {
            a.prefix("/admin")
             .site_title("Mon Application")
        })
        .build()
        .await
        .expect("Erreur de démarrage");

    app.run().await;
}
```
