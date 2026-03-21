# Application, Server & Database

## Application

| Variable | Default | Description |
|----------|---------|-------------|
| `DEBUG` | `false` | Global dev/prod switch — read **once** at startup via `LazyLock`. Enables: `debug` log level, detailed error pages, admin template hot reload. In production (`false`): `warn` level, generic errors. |
| `BASE_DIR` | `.` | Application root directory |
| `PROJECT_NAME` | `myproject` | Project name (used for `root_urlconf`) |
| `TIME_ZONE` | `UTC` | Timezone (not yet implemented) |
| `DEFAULT_AUTO_FIELD` | — | Default auto field type for models |
| `LANG` | system locale | CLI language (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priority: `.env` > system locale (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Listening port |
| `SECRET_KEY` | `default_secret_key` | Secret key (CSRF, signatures) — **must be changed in production** |

---

## Database

### Connection

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_URL` | — | Full connection URL (takes priority over all component variables) |
| `DB_ENGINE` | `sqlite` | Engine: `postgres`, `mysql`, `mariadb`, `sqlite` |
| `DB_USER` | — | Username (required except for SQLite) |
| `DB_PASSWORD` | — | Password (required except for SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (default depends on engine) |
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

### SQL Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_LOGGING` | `false` | Enable SQL query logging (`true`, `1`, `yes`) |

---

## Redirects

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIRECT_ANONYMOUS` | `/` | Redirect URL for unauthenticated visitors |
| `LOGGING_URL` | `/` | Redirect URL to the login page |
| `USER_CONNECTED_URL` | `/` | Redirect URL after login |

---

## See also

| Section | Description |
| --- | --- |
| [Assets & media](/docs/en/env/assets) | Static files, media, templates |
| [Security & sessions](/docs/en/env/security) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
