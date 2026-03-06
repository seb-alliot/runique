# Exemples

```no_run
use runique::prelude::DatabaseConfig;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
// Depuis les variables d'environnement
let config = DatabaseConfig::from_env()?.build();
let db = config.connect().await?;

// Ou depuis une URL
let config = DatabaseConfig::from_url("sqlite://db.sqlite")?.build();
let db = config.connect().await?;
# Ok(())
# }
```
