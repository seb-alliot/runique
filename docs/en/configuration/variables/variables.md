# Environment Variables

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Server port |
| `DEBUG` | `false` | Debug mode (templates, logs, etc.) |

> **⚠️ Production:** Set `DEBUG=false` explicitly. In debug mode, detailed error pages are visible, potentially revealing sensitive information. Also, compiling with `cargo build --release` automatically disables debug assertions, but `DEBUG=true` may override this.

---

## Database

### Connection

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Full connection URL (takes priority over all component variables) |
| `DB_ENGINE` | `sqlite` | `postgres`, `mysql`, `mariadb`, `sqlite` |
| `DB_USER` | — | DB user (required except for SQLite) |
| `DB_PASSWORD` | — | DB password (required except for SQLite) |
| `DB_HOST` | `localhost` | DB host |
| `DB_PORT` | `5432` / `3306` | DB port (default depends on engine) |
| `DB_NAME` | `local_base.sqlite` | Database name |

### Connection pool

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_MAX_CONNECTIONS` | `100` | Maximum pool size |
| `DB_MIN_CONNECTIONS` | `20` | Minimum pool size |

### Timeouts

| Variable | Default | Unit | Description |
|----------|---------|------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | seconds | Connection establishment timeout |
| `DB_ACQUIRE_TIMEOUT` | `500` | milliseconds | Pool acquire timeout |
| `DB_IDLE_TIMEOUT` | `300` | seconds | Idle connection lifetime |
| `DB_MAX_LIFETIME` | `3600` | seconds | Maximum connection lifetime |

### Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_LOGGING` | `false` | Enable SQL query logging (`true`, `1`, `yes`) |

**PostgreSQL (direct URL):**

```env
DATABASE_URL=postgres://user:password@localhost:5432/dbname
```

**PostgreSQL (component variables):**

```env
DB_ENGINE=postgres
DB_USER=postgres
DB_PASSWORD=secret
DB_HOST=localhost
DB_PORT=5432
DB_NAME=runique
```

**SQLite (dev):**

```env
DB_ENGINE=sqlite
DB_NAME=runique.db
```

**Custom pool:**

```env
DB_MAX_CONNECTIONS=50
DB_MIN_CONNECTIONS=5
DB_CONNECT_TIMEOUT=5
DB_IDLE_TIMEOUT=600
DB_LOGGING=true
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
| `SECRET_KEY` | *(required)* | CSRF secret key (⚠️ CHANGE IN PROD!) |

```env
SECRET_KEY=your_secret_key_change_this_in_production
```

> Allowed hosts are configured in the builder (`main.rs`), not via an environment variable:
>
> ```rust
> .with_allowed_hosts(|h| h.enabled(true).host("example.com"))
> ```

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
SECRET_KEY=your_secret_key_here_change_in_production
```

---

## Generate a secret key

```bash
# Python
python -c "import secrets; print(secrets.token_urlsafe(32))"

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
SECRET_KEY=<dynamically generated>
DATABASE_URL=postgres://user:pwd@prod-db.example.com:5432/runique
```

### Development

```env
DEBUG=true
PORT=3000
IP_SERVER=127.0.0.1
SECRET_KEY=any_dev_key
DATABASE_URL=sqlite:runique.db?mode=rwc
```

### Testing

```env
DEBUG=true
SECRET_KEY=test_key
DATABASE_URL=sqlite::memory:
```

---

## See also

| Section | Description |
| --- | --- |
| [Accessing config in code](/docs/en/configuration/code) | `RuniqueConfig`, validation |
| [Builder](/docs/en/configuration/builder) | Programmatic configuration |

## Back to summary

- [Configuration](/docs/en/configuration)
