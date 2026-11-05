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
           })
         .with_allowed_hosts(|h| {
              h.host("monsite.fr")
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
    m.with_csp(|c| c.policy(SecurityPolicy::strict()))
     .with_allowed_hosts(|h| h.host("monsite.fr"))
})
```

> `RUNIQUE_ENV` fait le reste : avec `dev`, les hôtes ne sont pas validés et `upgrade-insecure-requests` est retiré ; avec `prod`, les deux s'appliquent. Pour votre propre code, `is_dev()` donne la même information.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF et chemins exemptés |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Hosts & cache](/docs/fr/middleware/hosts-cache) | Validation des hosts |
| [CORS](/docs/fr/middleware/cors) | Cross-Origin Resource Sharing |
| [Open Redirect](/docs/fr/middleware/open-redirect) | Protection open redirect |
| [Permissions-Policy](/docs/fr/middleware/permissions-policy) | Permissions APIs navigateur |
| [Trusted Proxies](/docs/fr/middleware/trusted-proxies) | Extraction de l'IP cliente réelle |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
