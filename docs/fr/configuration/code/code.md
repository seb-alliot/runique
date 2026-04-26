# Accès à la configuration dans le code

## Charger la configuration

```rust
use runique::config::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.server.port);
```

---

## Accès dans un handler

```rust
async fn my_handler(mut request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Secret key: {}", config.server.secret_key);
}
```

---

## Configuration conditionnelle

```rust
if request.engine.config.debug {
    // Mode debug: logs détaillés, templates rechargés
} else {
    // Mode production: cache templates, pas de logs sensibles
}
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](/docs/fr/configuration/variables) | Toutes les variables `.env` |
| [Builder](/docs/fr/configuration/builder) | Configuration programmatique |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
