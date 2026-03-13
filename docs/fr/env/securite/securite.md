# Sécurité, Middlewares, CSP & Sessions

## Sécurité

| Variable | Défaut | Description |
|----------|--------|-------------|
| `ALLOWED_HOSTS` | `*` | Hosts autorisés (séparés par virgule) |
| `SANITIZE_INPUTS` | `true` | Activer la sanitisation des entrées utilisateur |
| `STRICT_CSP` | `false` | CSP stricte |
| `ENFORCE_HTTPS` | `false` | Forcer HTTPS |
| `RATE_LIMITING` | `false` | Activer la limitation de débit |

---

## Middlewares

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_ENABLE_CSP` | `true` (prod) / `false` (dev) | Activer les headers Content Security Policy |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | `true` (prod) / `false` (dev) | Valider le header Host contre `ALLOWED_HOSTS` |
| `RUNIQUE_ENABLE_DEBUG_ERRORS` | `false` (prod) / `true` (dev) | Pages d'erreur détaillées |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | Headers de cache HTTP |
| `RUNIQUE_ALLOWED_HOSTS` | `*` | Hosts autorisés pour le middleware de validation |

---

## CSP (Content Security Policy)

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_POLICY_CSP_DEFAULT` | — | Sources autorisées pour `default-src` (séparées par espace) |
| `RUNIQUE_POLICY_CSP_SCRIPTS` | — | Sources autorisées pour `script-src` |
| `RUNIQUE_POLICY_CSP_STYLES` | — | Sources autorisées pour `style-src` |
| `RUNIQUE_POLICY_CSP_IMAGES` | — | Sources autorisées pour `img-src` |
| `RUNIQUE_POLICY_CSP_FONTS` | — | Sources autorisées pour `font-src` |
| `RUNIQUE_POLICY_CSP_STRICT_NONCE` | `false` | Activer le nonce strict pour les scripts inline |

---

## Rate Limiting

| Variable                        | Défaut | Description                                               |
|---------------------------------|--------|-----------------------------------------------------------|
| `RUNIQUE_RATE_LIMIT_REQUESTS`   | `60`   | Nombre de requêtes autorisées par fenêtre (`RateLimiter`) |
| `RUNIQUE_RATE_LIMIT_WINDOW_SECS`| `60`   | Durée de la fenêtre en secondes (`RateLimiter`)           |

Voir [Rate Limiting](https://github.com/seb-alliot/runique/blob/main/docs/fr/middleware/rate-limit/rate-limit.md) pour l'utilisation.

---

## LoginGuard

| Variable                       | Défaut | Description                                                    |
|--------------------------------|--------|----------------------------------------------------------------|
| `RUNIQUE_LOGIN_MAX_ATTEMPTS`   | `5`    | Nombre d'échecs avant verrouillage du compte (`LoginGuard`)    |
| `RUNIQUE_LOGIN_LOCKOUT_SECS`   | `300`  | Durée du verrouillage en secondes (`LoginGuard`)               |

Voir [LoginGuard](https://github.com/seb-alliot/runique/blob/main/docs/fr/auth/login-guard/login-guard.md) pour l'utilisation.

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
