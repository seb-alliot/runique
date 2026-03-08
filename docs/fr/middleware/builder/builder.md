# Configuration du Builder

`RuniqueApp::builder(config)` est l'unique point d'entrée. Toute la configuration des middlewares passe par `.middleware(|m| { ... })`.

## Exemple complet

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .with_error_handler(true)
    .middleware(|m| {
        m.with_csp(true)               // CSP & headers sécurité
         .with_host_validation(true)   // Validation des hosts
         .with_cache(true)             // No-cache en dev
    })
    .statics()
    .build()
    .await?;
```

---

## Personnaliser les middlewares

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.with_csp(false)              // Désactiver CSP
         .with_host_validation(false)  // Désactiver la validation des hosts
    })
    .build()
    .await?;
```

> En mode `DEBUG=true`, `with_csp` et `with_host_validation` sont déjà désactivés par défaut.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Protection CSRF |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md) | Content Security Policy |
| [Hosts & cache](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/hosts-cache/hosts-cache.md) | Validation des hosts |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
