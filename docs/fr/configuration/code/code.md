# Accès à la configuration dans le code

## Charger la configuration

```rust
use runique::config::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Debug: {}", config.debug);
println!("Port: {}", config.server.port);
```

`RuniqueConfig::from_env()` lit le `.env` et l'environnement une fois, au démarrage. Ses principaux champs :

| Champ | Contenu |
| --- | --- |
| `config.debug` | `DEBUG` |
| `config.server` | `ip_server`, `port`, `secret_key`, `public_url` (posé par `.with_public_url(...)`) |
| `config.static_files` | Dossiers et préfixes : templates, statiques, médias |
| `config.security` | HTTPS, HSTS, ACME |
| `config.password` | La `PasswordConfig` |
| `config.timezone` | `TZ` |

---

## Accès dans un handler

```rust
async fn my_handler(mut request: Request) -> AppResult<Response> {
    let config = &request.engine.config;

    context_update!(request => {
        "debug" => config.debug,
        "port" => config.server.port,
    });
    request.render("page.html")
}
```

> Ne passez jamais `config.server.secret_key` à un template ni à un log : il signe les jetons CSRF et les sessions.

---

## Configuration conditionnelle

```rust
if request.engine.config.debug {
    // Mode debug : pages d'erreur détaillées, templates relus à chaque requête
} else {
    // Mode production : erreurs génériques
}
```

Hors d'un handler, `is_debug()` (dans le prelude) donne la même information.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Fichier .env](/docs/fr/configuration/variables) | Exemples complets |
| [Builder](/docs/fr/configuration/builder) | Configuration programmatique |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
