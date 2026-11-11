# Exemple d'utilisation du builder

```rust,no_run
use runique::prelude::*;

async fn index(mut request: Request) -> AppResult<Response> {
    request.render("index.html")
}

fn routes() -> Routes {
    urlpatterns! {
        "/" => view! { index }, name = "index",
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration lue dans les variables d'environnement (.env)
    let config = RuniqueConfig::from_env();

    // Connexion à la base, établie avant le build
    let db = DatabaseConfig::from_env()?.build().connect().await?;

    RuniqueAppBuilder::new(config)
        .routes(routes())
        .with_database(db)
        .statics()
        .middleware(|m| m.with_csp(|c| c.policy(SecurityPolicy::strict())))
        .build()
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { Box::new(e) })?
        .run()
        .await?;

    Ok(())
}
```

Chaque méthode du builder ne peut être appelée qu'une fois : un second appel ne compile pas.
Le panel d'administration se branche avec `.with_admin(…)` (feature `admin`, voir `AdminStaging`).
