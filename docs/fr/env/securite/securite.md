# Sécurité, Middlewares, CSP & Sessions

## Middlewares

| Variable | Défaut | Description |
| --- | --- | --- |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | Headers de cache HTTP |

> **CSP** — Configurée exclusivement via le builder (`.with_csp(...)`). Voir [CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md).
> **Host validation** — Configurée exclusivement via le builder (`.with_allowed_hosts([...])`). Voir [Host Validation](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/hosts/hosts.md).

---

## Sessions

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Intervalle du cleanup périodique (secondes) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 Mo) | Seuil de cleanup proactif des sessions anonymes expirées (octets) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 Mo) | Seuil d'urgence — cleanup synchrone + 503 si dépassé (octets) |

Voir [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md) pour le détail du comportement.

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/application/application.md) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Assets & médias](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/assets/assets.md) | Fichiers statiques, médias, templates |

## Retour au sommaire

- [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)
