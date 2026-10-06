# Sécurité, Middlewares, CSP & Sessions

## Middlewares

Aucun middleware ne se configure par le `.env`. En mode debug (`DEBUG=true`), des en-têtes no-cache sont ajoutés sur `localhost` ; `.middleware(|m| m.with_cache(true))` les désactive.

> **CSP** — Configurée exclusivement via le builder (`.with_csp(...)`). Voir [CSP](/docs/fr/middleware/csp).
> **Host validation** — Configurée exclusivement via le builder (`.with_allowed_hosts(|h| h.enabled(true).host("..."))`). Voir [Host Validation](/docs/fr/middleware/hosts-cache).

---

## Sessions

Les limites mémoire et l'intervalle de cleanup sont configurés via le builder — voir [Sessions](/docs/fr/session).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | DEBUG, IP_SERVER, PORT, DB |
| [Assets & médias](/docs/fr/env/assets) | Fichiers statiques, médias, templates |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
