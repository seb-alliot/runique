# Exemples

```no_run
use runique::prelude::DatabaseConfig;
use std::time::Duration;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = DatabaseConfig::from_url("postgres://user:pass@localhost/db")?
    .max_connections(50)
    .connect_timeout(Duration::from_secs(10))
    .build();

let db = config.connect().await?;
# Ok(())
# }
```
