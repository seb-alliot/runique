# Content Security Policy (CSP)

Runique applique une politique CSP **par défaut, sans configuration** — le middleware de sécurité (headers + CSP) est actif sur toutes les réponses, même sans jamais appeler `.with_csp(...)`. Un nonce unique est généré par requête et injecté dans les templates Tera.

---

## Table des matières

| Section | Description |
| --- | --- |
| [Profils CSP](/docs/fr/middleware/csp-profils) | `default()`, `strict()`, `permissive()` — comparaison et cas d'usage |
| [Directives](/docs/fr/middleware/csp-directives) | Toutes les directives configurables |
| [Nonce CSP](/docs/fr/middleware/csp-nonce) | Fonctionnement du nonce, usage dans les templates |
| [Headers de sécurité](/docs/fr/middleware/csp-headers) | Tous les headers injectés automatiquement |

---

## Démarrage rapide

Sans rien configurer, la CSP par défaut (`SecurityPolicy::default()`) et tous les headers de sécurité (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Permissions-Policy, COEP/COOP/CORP, HSTS si HTTPS réel) sont déjà envoyés sur chaque réponse. `.with_csp(...)` ne les **active** pas — il remplace la politique par défaut par la vôtre :

```rust
RuniqueApp::builder(config)
    .middleware(|m| {
        m.with_csp(|c| c.policy(SecurityPolicy::strict()))
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

La directive `ENFORCE_HTTPS=true` active une redirection **308** (`Redirect::permanent()`) vers HTTPS pour toutes les requêtes HTTP. Cette redirection repose sur le header `X-Forwarded-Proto` pour détecter si la requête arrive en HTTP ou HTTPS.

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
