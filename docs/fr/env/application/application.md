# Application, Serveur & Base de données

## Application

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DEBUG` | `false` | Interrupteur global dev/prod — lu **une seule fois** au démarrage via `LazyLock`. Active : niveau de log `debug`, pages d'erreur détaillées, hot reload templates admin. En production (`false`) : niveau `warn`, erreurs génériques. |
| `BASE_DIR` | `.` | Répertoire racine de l'application |
| `PROJECT_NAME` | `myproject` | Nom du projet (utilisé pour `root_urlconf`) |
| `TIME_ZONE` | `UTC` | Fuseau horaire (pas encore implémenté) |
| `DEFAULT_AUTO_FIELD` | — | Type de champ auto par défaut pour les modèles |
| `LANG` | locale système | Langue de la CLI (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priorité : `.env` > locale système (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP d'écoute |
| `PORT` | `3000` | Port d'écoute |
| `SECRET_KEY` | `default_secret_key` | Clé secrète (CSRF, signatures) — **à changer en production** |

---

## Base de données

### Connexion

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_URL` | — | URL complète (prioritaire sur toutes les variables composantes) |
| `DB_ENGINE` | `sqlite` | Moteur : `postgres`, `mysql`, `mariadb`, `sqlite` |
| `DB_USER` | — | Utilisateur (requis sauf SQLite) |
| `DB_PASSWORD` | — | Mot de passe (requis sauf SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (défaut selon le moteur) |
| `DB_NAME` | `local_base.sqlite` | Nom de la base de données |

### Pool de connexions

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_MAX_CONNECTIONS` | `100` | Taille maximale du pool |
| `DB_MIN_CONNECTIONS` | `20` | Taille minimale du pool |

### Timeouts

| Variable | Défaut | Unité | Description |
|----------|--------|-------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | secondes | Timeout d'établissement de connexion |
| `DB_ACQUIRE_TIMEOUT` | `500` | millisecondes | Timeout d'acquisition depuis le pool |
| `DB_IDLE_TIMEOUT` | `300` | secondes | Durée d'inactivité avant fermeture |
| `DB_MAX_LIFETIME` | `3600` | secondes | Durée de vie maximale d'une connexion |

### Logging SQL

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_LOGGING` | `false` | Active les logs SQL (`true`, `1`, `yes`) |

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
| [Assets & médias](/docs/fr/env/assets) | Fichiers statiques, médias, templates |
| [Sécurité & sessions](/docs/fr/env/securite) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Retour au sommaire

- [Variables d'environnement](/docs/fr/env)
