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
.with_log(|l| l.middleware(|m| m.host_validation(LogLevel::WARN)))
```

En production, `WARN` est recommandé pour détecter des attaques ou des mauvaises configurations.

### Configuration via le builder

La validation des hosts se configure dans `main.rs` via le builder — la liste ne passe pas par une variable d'environnement :

```rust
.middleware(|m| {
    m.with_allowed_hosts(|h| {
        h.host("example.com")
         .host(".example.com")  // match example.com ET tous ses sous-domaines
    })
})
```

### Patterns supportés

- `"localhost"` — match exact
- `".example.com"` — match `example.com` et `*.example.com`
- `"*"` — tous les hosts (⚠️ dangereux en production)

La comparaison ignore la casse et le point final : `Example.COM` et `example.com.` désignent le même host que `example.com`. Un host vide n'est jamais accepté.

### Selon l'environnement

La validation est active avec `RUNIQUE_ENV=prod` dès qu'un hôte est déclaré, inactive avec `RUNIQUE_ENV=dev`. `.enabled(true)` ou `.enabled(false)` a le dernier mot, par exemple pour essayer la validation en local.

---

## Cache-Control

### Mode développement (`RUNIQUE_ENV=dev` **et** accès via `localhost`/`127.0.0.1`/`[::1]`)

Les deux conditions sont requises — un accès en `RUNIQUE_ENV=dev` via un autre nom d'hôte (LAN, tunnel, staging) ne déclenche pas ces headers. Ajoutés pour forcer le rechargement :

```http
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
Expires: 0
```

### Mode production (`RUNIQUE_ENV=prod`)

Aucun en-tête no-cache n'est ajouté : le navigateur applique son cache habituel. Le cache des fichiers statiques et médias se règle avec `.static_files(|s| s.static_cache("…").media_cache("…"))`.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](/docs/fr/middleware/csp) | Content Security Policy |
| [Builder](/docs/fr/middleware/builder) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](/docs/fr/middleware)
