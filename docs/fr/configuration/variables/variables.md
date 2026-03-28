# Variables d'environnement

## Serveur

| Variable | Défaut | Description |
|----------|--------|-------------|
| `IP_SERVER` | `127.0.0.1` | Adresse IP écoute |
| `PORT` | `3000` | Port serveur |
| `DEBUG` | `true` | Mode debug (templates, logs, etc.) |

> **⚠️ Production :** Définir `DEBUG=false` explicitement. En mode debug, les pages d'erreur détaillées sont visibles, révélant potentiellement des informations sensibles. De plus, compiler avec `cargo build --release` désactive automatiquement les assertions debug, mais `DEBUG=true` peut outrepasser cela.

---

## Base de données

### Connexion

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_URL` | — | URL complète (prioritaire sur toutes les variables composantes) |
| `DB_ENGINE` | `sqlite` | `postgres`, `mysql`, `mariadb`, `sqlite` |
| `DB_USER` | — | Utilisateur DB (requis sauf SQLite) |
| `DB_PASSWORD` | — | Mot de passe DB (requis sauf SQLite) |
| `DB_HOST` | `localhost` | Host DB |
| `DB_PORT` | `5432` / `3306` | Port DB (défaut selon le moteur) |
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

### Logging

| Variable | Défaut | Description |
|----------|--------|-------------|
| `DB_LOGGING` | `false` | Active les logs SQL (`true`, `1`, `yes`) |

**PostgreSQL (URL directe) :**

```env
DB_URL=postgres://user:password@localhost:5432/dbname
```

**PostgreSQL (variables composantes) :**

```env
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev) :**

```env
DB_ENGINE=sqlite
DB_NAME=runique.db
```

**Pool personnalisé :**

```env
DB_MAX_CONNECTIONS=50
DB_MIN_CONNECTIONS=5
DB_CONNECT_TIMEOUT=5
DB_IDLE_TIMEOUT=600
DB_LOGGING=true
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

```env
SECRETE_KEY=your_secret_key_change_this_in_production
```

> La validation des hosts autorisés se configure dans le builder (`main.rs`), pas via une variable d'environnement :
>
> ```rust
> .with_allowed_hosts(|h| h.enabled(true).host("example.com"))
> ```

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
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Développement

```env
DEBUG=true
PORT=3000
IP_SERVER=127.0.0.1
SECRETE_KEY=any_dev_key
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Testing

```env
DEBUG=true
SECRETE_KEY=test_key
DATABASE_URL=sqlite::memory:
```

---

## Voir aussi

| Section | Description |
| --- | --- |
| [Accès dans le code](/docs/fr/configuration/code) | `RuniqueConfig`, validation |
| [Builder](/docs/fr/configuration/builder) | Builder classique et Intelligent |

## Retour au sommaire

- [Configuration](/docs/fr/configuration)
