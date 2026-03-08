# Accès à la configuration dans le code

## Charger la configuration

```rust
use runique::config_runique::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.port);
println!("DB: {}", config.database_url);
```

---

## Accès dans un handler

```rust
use runique::config_runique::RuniqueConfig;

async fn my_handler(template: TemplateContext) -> Response {
    let config = &template.engine.config;

    println!("Debug mode: {}", config.debug);
    println!("Port: {}", config.server.port);
    println!("IP: {}", config.server.ip_server);
    println!("Allowed hosts: {:?}", config.security.allowed_hosts);
    println!("Secret key: {}", config.security.secrete_key);
}
```

---

## Configuration conditionnelle

```rust
if template.engine.config.debug {
    // Mode debug: logs détaillés, templates rechargés
} else {
    // Mode production: cache templates, pas de logs sensibles
}

if template.engine.config.security.allowed_hosts.contains("*") {
    // ⚠️ Attention: tous les hosts sont autorisés (danger en production!)
}
```

---

## Validation de configuration

La configuration est validée au startup :

```rust
let config = RuniqueConfig::from_env()
    .expect("Configuration invalide");

// Retourne Err() si :
// - DATABASE_URL manquant
// - SECRETE_KEY manquant
// - Variables invalides
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/variables/variables.md) | Toutes les variables `.env` |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/builder/builder.md) | Configuration programmatique |

## Retour au sommaire

- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)
