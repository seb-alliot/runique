# Configuration du Builder

`RuniqueApp::builder(config)` est l'unique point d'entrée. Toute la configuration des middlewares passe par `.middleware(|m| { ... })`.

## Exemple complet

```rust
let app = RuniqueApp::builder(config)
    .routes(url::routes())
    .with_database(db)
    .middleware(|m| {
        m.with_csp(|c| {
              c.policy(SecurityPolicy::strict())
               .with_header_security(true)
           })
         .with_allowed_hosts(|h| {
              h.enabled(true)
               .host("monsite.fr")
               .host("www.monsite.fr")
           })
         .with_cache(true)
    })
    .statics()
    .build()
    .await?;
```

---

## Conditionnel selon l'environnement

```rust
.middleware(|m| {
    m.with_csp(|c| {
          c.policy(SecurityPolicy::strict())
           .with_upgrade_insecure(!is_debug())
       })
     .with_allowed_hosts(|h| {
          h.enabled(!is_debug())  // désactivé en dev, actif en prod
           .host("monsite.fr")
       })
})
```

> En mode `DEBUG=true`, `is_debug()` retourne `true` — les guards de sécurité peuvent être désactivés conditionnellement.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Hosts & cache](/docs/fr/middleware/hosts-cache) | Validation des hosts |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
