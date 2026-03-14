# Validation des hosts & Cache-Control

## Validation des Hosts (Allowed Hosts)

### Fonctionnement

- Compare le header `Host` de la requête contre `ALLOWED_HOSTS`
- Bloque les requêtes avec un host non autorisé (HTTP 400)
- Protection contre les attaques Host Header Injection

### Configuration `.env`

```env
# Hosts autorisés (séparés par des virgules)
ALLOWED_HOSTS=localhost,127.0.0.1,example.com

# Patterns supportés :
# localhost       → match exact
# .example.com   → match example.com ET *.example.com
# *              → TOUS les hosts (⚠️ DANGEREUX en production !)
```

### Mode debug

En `DEBUG=true`, la validation des hosts est **désactivée par défaut** pour faciliter le développement.

---

## Cache-Control

### Mode développement (`DEBUG=true`)

Headers `no-cache` ajoutés pour forcer le rechargement :

```
Cache-Control: no-cache, no-store, must-revalidate
Pragma: no-cache
```

### Mode production (`DEBUG=false`)

Headers de cache activés pour les performances.

---

## Variables d'environnement liées à la sécurité

| Variable | Défaut | Description |
|----------|--------|-------------|
| `SECRETE_KEY` | *(requis)* | Clé secrète pour le CSRF |
| `ALLOWED_HOSTS` | `*` | Hosts autorisés |
| `DEBUG` | `true` | Mode debug (affecte cache, hosts) |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | *(auto)* | Force la validation des hosts |
| `RUNIQUE_ENABLE_CACHE` | *(auto)* | Force le contrôle cache |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [CSP & headers](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md) | Content Security Policy |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/builder/builder.md) | Configuration du builder |

## Retour au sommaire

- [Middleware & Sécurité](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/08-middleware.md)
