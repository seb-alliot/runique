# Sécurité, Middlewares, CSP & Sessions

## Middlewares

| Variable | Défaut | Description |
| --- | --- | --- |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | Headers de cache HTTP |

> **CSP** — Configurée exclusivement via le builder (`.with_csp(...)`). Voir [CSP](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/csp/csp.md).
> **Host validation** — Configurée exclusivement via le builder (`.with_allowed_hosts([...])`). Voir [Host Validation](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/hosts/hosts.md).

---

## Sessions

Les limites mémoire et l'intervalle de cleanup sont configurés via le builder — voir [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/session/14-sessions.md).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/application/application.md) | DEBUG, IP_SERVER, PORT, DB, Redirections |
| [Assets & médias](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/assets/assets.md) | Fichiers statiques, médias, templates |

## Retour au sommaire

- [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)
