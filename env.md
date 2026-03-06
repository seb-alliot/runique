# Environment Variables — Runique

All configurable keys via `.env`. Values shown are defaults applied when the variable is absent.

---

## Application

| Variable | Default | Description |
|----------|---------|-------------|
| `DEBUG` | `false` (release) / `true` (debug_assertions) | Debug mode — enables detailed error pages |
| `BASE_DIR` | `.` | Application root directory |
| `PROJECT_NAME` | `myproject` | Project name (used for `root_urlconf`) |
| `LANGUAGE_APP` | `en-us` | Application language code |
| `TIME_ZONE` | `UTC` | Timezone |
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
| `DB_NAME` | `runique_db` | Database name (used if DATABASE_URL is absent) |

---

## Static Files & Media

| Variable | Default | Description |
|----------|---------|-------------|
| `STATICFILES_DIRS` | `static` | Static files directory |
| `STATIC_URL` | `/static` | URL prefix for static files |
| `MEDIA_ROOT` | `media` | Uploaded media files directory |
| `MEDIA_URL` | `/media` | URL prefix for media files |
| `STATIC_RUNIQUE_PATH` | — | Path to Runique internal assets |
| `STATIC_RUNIQUE_URL` | `/runique/static` | URL prefix for Runique internal assets |
| `MEDIA_RUNIQUE_PATH` | — | Path to Runique internal media |
| `MEDIA_RUNIQUE_URL` | `/runique/media` | URL prefix for Runique internal media |
| `TEMPLATES_DIR` | — | Tera templates directory |
| `TEMPLATES_RUNIQUE` | — | Runique internal templates directory |
| `STATICFILES` | `default_storage` | Storage backend |

---

## Security

| Variable | Default | Description |
|----------|---------|-------------|
| `ALLOWED_HOSTS` | `*` | Allowed hosts (comma-separated) |
| `SANITIZE_INPUTS` | `true` | Enable user input sanitization |
| `STRICT_CSP` | `false` | Strict CSP mode |
| `ENFORCE_HTTPS` | `false` | Force HTTPS |
| `RATE_LIMITING` | `false` | Enable rate limiting |

---

## Middlewares

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_ENABLE_CSP` | `true` (prod) / `false` (dev) | Enable Content Security Policy headers |
| `RUNIQUE_ENABLE_HOST_VALIDATION` | `true` (prod) / `false` (dev) | Validate Host header against ALLOWED_HOSTS |
| `RUNIQUE_ENABLE_DEBUG_ERRORS` | `false` (prod) / `true` (dev) | Detailed error pages |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | HTTP cache headers |
| `RUNIQUE_ALLOWED_HOSTS` | `*` | Allowed hosts for the host validation middleware |

---

## CSP (Content Security Policy)

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_POLICY_CSP_DEFAULT` | — | Allowed sources for `default-src` (space-separated) |
| `RUNIQUE_POLICY_CSP_SCRIPTS` | — | Allowed sources for `script-src` |
| `RUNIQUE_POLICY_CSP_STYLES` | — | Allowed sources for `style-src` |
| `RUNIQUE_POLICY_CSP_IMAGES` | — | Allowed sources for `img-src` |
| `RUNIQUE_POLICY_CSP_FONTS` | — | Allowed sources for `font-src` |
| `RUNIQUE_POLICY_CSP_STRICT_NONCE` | `false` | Enable strict nonce for inline scripts |

---

## Sessions

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Periodic cleanup interval (seconds) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 MB) | Proactive cleanup threshold — background purge of expired anonymous sessions (bytes) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 MB) | Emergency threshold — synchronous cleanup + 503 if still exceeded (bytes) |

---

## Redirects

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIRECT_ANONYMOUS` | `/` | Redirect URL for unauthenticated visitors |
| `LOGGING_URL` | `/` | Redirect URL to the login page |
| `USER_CONNECTED_URL` | `/` | Redirect URL after login |
