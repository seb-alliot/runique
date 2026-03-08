# Variables d'environnement

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP écoute |
| `PORT` | `3000` | Port serveur |
| `DEBUG` | `true` | Mode debug (templates, logs, etc.) |

---

## Base de données

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DATABASE_URL` | — | Chaîne de connexion complète |
| `DB_ENGINE` | `postgres` | `postgres`, `sqlite`, `mysql` |
| `DB_USER` | `postgres` | Utilisateur DB |
| `DB_PASSWORD` | — | Mot de passe DB |
| `DB_HOST` | `localhost` | Host DB |
| `DB_PORT` | `5432` | Port DB |
| `DB_NAME` | `runique` | Nom base de données |

**PostgreSQL :**

```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev) :**

```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

---

## Templates & Assets

| Variable | Défaut | Description |
|----------|--------|-------------|
| `TEMPLATES_DIR` | `templates` | Répertoire templates |
| `STATICFILES_DIRS` | `static` | Répertoire assets statiques |
| `MEDIA_ROOT` | `media` | Répertoire médias (uploads) |

```env
TEMPLATES_DIR=templates
STATICFILES_DIRS=static:demo-app/static
MEDIA_ROOT=uploads
```

---

## Sécurité

| Variable | Défaut | Description |
|----------|--------|-------------|
| `SECRETE_KEY` | *(requis)* | Clé secrète CSRF (⚠️ CHANGE EN PROD!) |
| `ALLOWED_HOSTS` | `*` | Hosts autorisés (comma-separated) |

**ALLOWED_HOSTS patterns :**

- `localhost` — exact match
- `*` — wildcard tous les hosts (DANGER en production !)
- `.example.com` — match `example.com` et `*.example.com`

```env
SECRETE_KEY=your_secret_key_change_this_in_production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

---

## Fichier .env complet

```env
# ============================================================================
# SERVER CONFIGURATION
# ============================================================================
IP_SERVER=127.0.0.1
PORT=3000
DEBUG=true

# ============================================================================
# DATABASE CONFIGURATION
# ============================================================================
DATABASE_URL=postgres://postgres:password@localhost:5432/runique
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=password
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique

# SQLite (Development only)
# DATABASE_URL=sqlite:runique.db?mode=rwc

# ============================================================================
# TEMPLATES & STATIC FILES
# ============================================================================
TEMPLATES_DIR=templates
STATICFILES_DIRS=static
MEDIA_ROOT=media

# ============================================================================
# SECURITY
# ============================================================================
SECRETE_KEY=your_secret_key_here_change_in_production
ALLOWED_HOSTS=localhost,127.0.0.1
```

---

## Générer une clé secrète

```bash
# Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# OpenSSL
openssl rand -base64 32
```

---

## Modes d'environnement

### Production

```env
DEBUG=false
PORT=443
IP_SERVER=0.0.0.0
SECRETE_KEY=<généré dynamiquement>
ALLOWED_HOSTS=example.com,www.example.com,.api.example.com
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Développement

```env
DEBUG=true
PORT=3000
IP_SERVER=127.0.0.1
SECRETE_KEY=any_dev_key
ALLOWED_HOSTS=*
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Testing

```env
DEBUG=true
SECRETE_KEY=test_key
ALLOWED_HOSTS=localhost,127.0.0.1
DATABASE_URL=sqlite::memory:
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Accès dans le code](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/code/code.md) | `RuniqueConfig`, validation |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/builder/builder.md) | Builder classique et Intelligent |

## Retour au sommaire

- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/fr/configuration/03-configuration.md)
