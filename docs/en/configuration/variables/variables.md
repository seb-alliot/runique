# Environment Variables

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Server port |
| `DEBUG` | `true` | Debug mode (templates, logs, etc.) |

---

## Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Full connection string |
| `DB_ENGINE` | `postgres` | `postgres`, `sqlite`, `mysql` |
| `DB_USER` | `postgres` | DB user |
| `DB_PASSWORD` | — | DB password |
| `DB_HOST` | `localhost` | DB host |
| `DB_PORT` | `5432` | DB port |
| `DB_NAME` | `runique` | Database name |

**PostgreSQL:**

```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev):**

```env
DATABASE_URL=sqlite:runique.db?mode=rwc
```

---

## Templates & Assets

| Variable | Default | Description |
|----------|---------|-------------|
| `TEMPLATES_DIR` | `templates` | Templates directory |
| `STATICFILES_DIRS` | `static` | Static assets directory |
| `MEDIA_ROOT` | `media` | Media directory (uploads) |

```env
TEMPLATES_DIR=templates
STATICFILES_DIRS=static:demo-app/static
MEDIA_ROOT=uploads
```

---

## Security

| Variable | Default | Description |
|----------|---------|-------------|
| `SECRETE_KEY` | *(required)* | CSRF secret key (⚠️ CHANGE IN PROD!) |
| `ALLOWED_HOSTS` | `*` | Allowed hosts (comma-separated) |

**ALLOWED_HOSTS patterns:**

- `localhost` — exact match
- `*` — wildcard all hosts (DANGEROUS in production!)
- `.example.com` — matches `example.com` and `*.example.com`

```env
SECRETE_KEY=your_secret_key_change_this_in_production
ALLOWED_HOSTS=localhost,127.0.0.1,example.com,.api.example.com
```

---

## Complete .env file

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

## Generate a secret key

```bash
# Python
python3 -c "import secrets; print(secrets.token_urlsafe(32))"

# OpenSSL
openssl rand -base64 32
```

---

## Environment modes

### Production

```env
DEBUG=false
PORT=443
IP_SERVER=0.0.0.0
SECRETE_KEY=<dynamically generated>
ALLOWED_HOSTS=example.com,www.example.com,.api.example.com
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Development

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

## See also

| Section | Description |
| --- | --- |
| [Accessing config in code](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/code/code.md) | `RuniqueConfig`, validation |
| [Builder](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/builder/builder.md) | Programmatic configuration |

## Back to summary

- [Configuration](https://github.com/seb-alliot/runique/blob/main/docs/en/configuration/03-configuration.md)
