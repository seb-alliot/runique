# Sécurité, Middlewares, CSP & Sessions

## Middlewares

Aucun middleware ne se configure par le `.env`. En développement (`RUNIQUE_ENV=dev`), des en-têtes no-cache sont ajoutés sur `localhost` ; `.middleware(|m| m.with_cache(true))` les désactive.

> **CSP** — Configurée exclusivement via le builder (`.with_csp(...)`). Voir [CSP](/docs/fr/middleware/csp).
> **Host validation** — Configurée exclusivement via le builder (`.with_allowed_hosts(|h| h.host("..."))`). Voir [Host Validation](/docs/fr/middleware/hosts-cache).

---

## HTTPS, ACME et journalisation

| Variable | Défaut | Description |
|----------|--------|-------------|
| `ENFORCE_HTTPS` | `false` | Redirige HTTP vers HTTPS derrière un proxy (voir [HTTPS forcé](/docs/fr/middleware/csp)) |
| `HSTS_MAX_AGE` | `31536000` (1 an) | Durée de l'en-tête `Strict-Transport-Security`, en secondes |
| `ACME_ENABLED` | `false` | Certificats Let's Encrypt gérés par Runique (feature `acme`) |
| `ACME_DOMAIN`, `ACME_EMAIL` | — | Requis quand `ACME_ENABLED=true` |
| `ACME_CERTS_DIR` | `./certs` | Dossier des certificats — chemin absolu en production |
| `RUST_LOG` | — | Filtre de logs (`tracing`), prioritaire sur le niveau par défaut (`debug` avec `RUNIQUE_ENV=dev`, `warn` sinon) |
| `RUNIQUE_LOG_FILE` | — | Ajoute un fichier de logs, sans recompiler |

Voir [Réseau & HTTP](/docs/fr/installation/network) pour ACME.

---

## Sessions

Les limites mémoire et l'intervalle de cleanup sont configurés via le builder — voir [Sessions](/docs/fr/session).

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Application & Serveur](/docs/fr/env/application) | RUNIQUE_ENV, DEBUG, IP_SERVER, PORT, DB |
| [Assets & médias](/docs/fr/env/assets) | Fichiers statiques, médias, templates |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
