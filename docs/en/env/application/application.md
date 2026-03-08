# Application, Server & Database

## Application

| Variable | Default | Description |
|----------|---------|-------------|
| `DEBUG` | `false` (release) / `true` (debug_assertions) | Debug mode — enables detailed error pages |
| `BASE_DIR` | `.` | Application root directory |
| `PROJECT_NAME` | `myproject` | Project name (used for `root_urlconf`) |
| `LANGUAGE_APP` | `en-us` | Application language code (fr and en for errors — i18n in progress) |
| `TIME_ZONE` | `UTC` | Timezone (not yet implemented) |
| `DEFAULT_AUTO_FIELD` | — | Default auto field type for models |

---

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Listening port |
| `SECRET_KEY` | `default_secret_key` | Secret key (CSRF, signatures) — **must be changed in production** |

---

## Database

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Connection URL (`sqlite://...`, `postgres://...`, `mysql://...`) |
| `DB_NAME` | `runique_db` | Database name (used if `DATABASE_URL` is absent) |

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
| [Assets & media](https://github.com/seb-alliot/runique/blob/main/docs/en/env/assets/assets.md) | Static files, media, templates |
| [Security & sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/env/security/security.md) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Back to summary

- [Environment Variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)
