# Content Security Policy (CSP)

Runique applique une politique CSP via le middleware de sécurité, configuré exclusivement via le builder. Un nonce unique est généré par requête et injecté dans les templates Tera.

---

## Table des matières

| Section | Description |
| --- | --- |
| [Profils CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/profils.md) | `default()`, `strict()`, `permissive()` — comparaison et cas d'usage |
| [Directives](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/directives.md) | Toutes les directives configurables |
| [Nonce CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/nonce.md) | Fonctionnement du nonce, usage dans les templates |
| [Headers de sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/headers.md) | Tous les headers injectés automatiquement |

---

## Démarrage rapide

La CSP est désactivée par défaut — elle s'active uniquement via le builder :

```rust
RuniqueApp::new()
    .middleware(|m| {
        m.with_csp(|c| c)
    })
    .build()
    .await?;
```

Pour personnaliser :

```rust
.middleware(|m| {
    m.with_csp(|c| {
        c.with_nonce(true)
         .scripts(vec!["'self'", "https://cdn.example.com"])
         .images(vec!["'self'", "data:"])
    })
})
```

Dans vos templates :

```html
<script {% csp_nonce %}>
    // Ce script est autorisé par le nonce CSP
    console.log("OK");
</script>
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csrf/csrf.md) | Protection CSRF |
| [Builder & configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
