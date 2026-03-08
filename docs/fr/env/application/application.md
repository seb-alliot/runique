# Application, Serveur & Base de données

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DEBUG` | `false` (release) / `true` (debug_assertions) | Mode debug — active les pages d'erreur détaillées |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `PROJECT_NAME` | `myproject` | Nom du projet (utilisé pour `root_urlconf`) |
| `LANGUAGE_APP` | `en-us` | Code langue de l'application (fr et en pour les erreurs — i18n en cours) |
| `TIME_ZONE` | `UTC` | Fuseau horaire (pas encore implémenté) |
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
| `DB_NAME` | `runique_db` | Nom de la base (utilisé si `DATABASE_URL` est absent) |

---

## Redirections

| Variable | Défaut | Description |
|----------|--------|-------------|
| `REDIRECT_ANONYMOUS` | `/` | URL de redirection pour les visiteurs non connectés |
| `LOGGING_URL` | `/` | URL de redirection vers la page de login |
| `USER_CONNECTED_URL` | `/` | URL de redirection après connexion |

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Assets & médias](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/assets/assets.md) | Fichiers statiques, médias, templates |
| [Sécurité & sessions](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/securite/securite.md) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](https://github.com/seb-alliot/runique/blob/main/docs/fr/env/15-env.md)
