# Accès à la configuration dans le code

## Charger la configuration

```rust
use runique::config::RuniqueConfig;

let config = RuniqueConfig::from_env();

println!("Développement : {}", config.dev);
println!("Port: {}", config.server.port);
```

`RuniqueConfig::from_env()` lit le `.env` et l'environnement une fois, au démarrage. Ses principaux champs :

| Champ | Contenu |
| --- | --- |
| `config.dev` | `RUNIQUE_ENV=dev` |
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
        "dev" => config.dev,
        "port" => config.server.port,
    });
    request.render("page.html")
}
```

> Ne passez jamais `config.server.secret_key` à un template ni à un log : il signe les jetons CSRF et les sessions.

---

## Configuration conditionnelle

```rust
if request.engine.config.dev {
    // Développement : templates relus à chaque requête, pas de contrôles au démarrage
} else {
    // Production
}
```

Hors d'un handler, `is_dev()` (dans le prelude) donne la même information. `config.debug` et `is_debug()` ne disent qu'une chose : les pages d'erreur détaillées sont-elles affichées.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Fichier .env](/docs/fr/configuration/variables) | Exemples complets |
| [Builder](/docs/fr/configuration/builder) | Configuration programmatique |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
