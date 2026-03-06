# Variables d'environnement — Runique

Toutes les clés configurables via `.env`. Les valeurs entre `[]` sont les défauts appliqués si la variable est absente.

---

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DEBUG` | `false` (release) / `true` (debug_assertions) | Mode debug — active les pages d'erreur détaillées |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `PROJECT_NAME` | `myproject` | Nom du projet (utilisé pour `root_urlconf`) |
| `LANGUAGE_APP` | `en-us` | Code langue de l'application |
| `TIME_ZONE` | `UTC` | Fuseau horaire |
| `DEFAULT_AUTO_FIELD` | — | Type de champ auto par défaut pour les modèles |

---

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP d'écoute |
| `PORT` | `3000` | Port d'écoute |
| `SECRET_KEY` | `default_secret_key` | Clé secrète (CSRF, signatures) — **à changer en production** |

---

## Base de données

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | — | URL de connexion (`sqlite://...`, `postgres://...`, `mysql://...`) |
| `DB_NAME` | `runique_db` | Nom de la base (utilisé si DATABASE_URL absent) |

---

## Fichiers statiques et médias

| Variable | Défaut | Description |
|----------|--------|-------------|
| `STATICFILES_DIRS` | `static` | Dossier des fichiers statiques |
| `STATIC_URL` | `/static` | Préfixe URL pour les fichiers statiques |
| `MEDIA_ROOT` | `media` | Dossier des fichiers médias uploadés |
| `MEDIA_URL` | `/media` | Préfixe URL pour les médias |
| `STATIC_RUNIQUE_PATH` | — | Chemin vers les assets internes de Runique |
| `STATIC_RUNIQUE_URL` | `/runique/static` | Préfixe URL pour les assets de Runique |
| `MEDIA_RUNIQUE_PATH` | — | Chemin vers les médias internes de Runique |
| `MEDIA_RUNIQUE_URL` | `/runique/media` | Préfixe URL pour les médias de Runique |
| `TEMPLATES_DIR` | — | Dossier des templates Tera |
| `TEMPLATES_RUNIQUE` | — | Dossier des templates internes de Runique |
| `STATICFILES` | `default_storage` | Backend de stockage |

---

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
| `RUNIQUE_ENABLE_HOST_VALIDATION` | `true` (prod) / `false` (dev) | Valider le header Host contre ALLOWED_HOSTS |
| `RUNIQUE_ENABLE_DEBUG_ERRORS` | `false` (prod) / `true` (dev) | Pages d'erreur détaillées |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | Headers de cache HTTP |
| `RUNIQUE_ALLOWED_HOSTS` | `*` | Hosts autorisés pour le middleware host validation |

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

## Sessions

| Variable | Défaut | Description |
|----------|--------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Intervalle du cleanup périodique (secondes) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 Mo) | Seuil de déclenchement du cleanup proactif (octets) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 Mo) | Seuil d'urgence — cleanup synchrone + 503 si dépassé (octets) |

---

## Redirections

| Variable | Défaut | Description |
|----------|--------|-------------|
| `REDIRECT_ANONYMOUS` | `/` | URL de redirection pour les visiteurs non connectés |
| `LOGGING_URL` | `/` | URL de redirection vers la page de login |
| `USER_CONNECTED_URL` | `/` | URL de redirection après connexion |
