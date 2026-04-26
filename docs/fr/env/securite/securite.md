# Sécurité, Middlewares, CSP & Sessions

## Middlewares

| Variable | Défaut | Description |
| --- | --- | --- |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | Headers de cache HTTP |

> **CSP** — Configurée exclusivement via le builder (`.with_csp(...)`). Voir [CSP](/docs/fr/middleware/csp).
> **Host validation** — Configurée exclusivement via le builder (`.with_allowed_hosts(|h| h.enabled(true).host("..."))`). Voir [Host Validation](/docs/fr/middleware/hosts).

---

## Sessions

Les limites mémoire et l'intervalle de cleanup sont configurés via le builder — voir [Sessions](/docs/fr/session).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Assets & médias](/docs/fr/env/assets) | Fichiers statiques, médias, templates |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
