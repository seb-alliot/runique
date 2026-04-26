# Validation des hosts & Cache-Control

## Validation des Hosts (Allowed Hosts)

### Fonctionnement

- Compare le header `Host` de la requête contre la liste des hosts autorisés
- Bloque les requêtes avec un host non autorisé (HTTP 400)
- Protection contre les attaques Host Header Injection

### HTTP/2 et le header `Host`

HTTP/2 n'envoie pas de header `Host` — il utilise le pseudo-header `:authority`.
Runique gère ce cas automatiquement : si le header `Host` est absent, le middleware
lit l'authority depuis `request.uri().authority()`.

Ce comportement couvre les connexions HTTP/1.1 directes, HTTP/2 (ex. ACME ou Cloudflare),
et les proxies inverses qui proxifient en HTTP/2.

### Logging

Les rejections peuvent être loggées via `RuniqueLog` :

```rust
.with_log(|l| l.host_validation(Level::WARN))
```

En production, `WARN` est recommandé pour détecter des attaques ou des mauvaises configurations.

### Configuration via le builder

La validation des hosts se configure dans `main.rs` via le builder — il n'y a pas de variable d'environnement pour cela :

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.enabled(!is_debug())
         .host("example.com")
         .host(".example.com")  // match example.com ET tous ses sous-domaines
    })
})
```

### Patterns supportés

- `"localhost"` — match exact
- `".example.com"` — match `example.com` et `*.example.com`
- `"*"` — tous les hosts (⚠️ dangereux en production)

### Mode debug

En `DEBUG=true`, on passe généralement `.enabled(!is_debug())` pour désactiver la validation en développement.

---

## Cache-Control

### Mode développement (`DEBUG=true`)

Headers `no-cache` ajoutés pour forcer le rechargement :

```http
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Mode production (`DEBUG=false`)

Headers de cache activés pour les performances.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
