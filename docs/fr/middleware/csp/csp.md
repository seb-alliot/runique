# Content Security Policy (CSP)

Runique applique une politique CSP via le middleware de sécurité, configuré exclusivement via le builder. Un nonce unique est généré par requête et injecté dans les templates Tera.

---

## Table des matières

| Section | Description |
| --- | --- |
| [Profils CSP](/docs/fr/middleware/csp) | `default()`, `strict()`, `permissive()` — comparaison et cas d'usage |
| [Directives](/docs/fr/middleware/csp) | Toutes les directives configurables |
| [Nonce CSP](/docs/fr/middleware/csp) | Fonctionnement du nonce, usage dans les templates |
| [Headers de sécurité](/docs/fr/middleware/csp) | Tous les headers injectés automatiquement |

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
<script {% csp %}>
    // Ce script est autorisé par le nonce CSP
    console.log("OK");
</script>
```

---

## HTTPS forcé (`enforce_https`)

La directive `ENFORCE_HTTPS=true` active une redirection 301 vers HTTPS pour toutes les requêtes HTTP. Cette redirection repose sur le header `X-Forwarded-Proto` pour détecter si la requête arrive en HTTP ou HTTPS.

> **⚠️ Prérequis proxy :** `enforce_https` fait confiance au header `X-Forwarded-Proto`. En l'absence d'un reverse proxy de confiance (nginx, Caddy, etc.) qui contrôle ce header, un attaquant peut forger `X-Forwarded-Proto: https` pour contourner la redirection.
>
> **En production**, placez toujours Runique derrière un reverse proxy qui contrôle ce header :
> supprime les headers `X-Forwarded-Proto` entrants des clients et injecte lui-même la valeur correcte (`https` ou `http`) selon la connexion réelle.

```env
# .env
ENFORCE_HTTPS=true
```

```nginx
# nginx — exemple de configuration correcte
proxy_set_header X-Forwarded-Proto $scheme;
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSRF](/docs/fr/middleware/csrf) | Protection CSRF |
| [Builder & configuration](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
