# Security, Middlewares, CSP & Sessions

## Middlewares

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_ENABLE_HOST_VALIDATION` | `true` (prod) / `false` (dev) | Validate Host header against `ALLOWED_HOSTS` |
| `RUNIQUE_ENABLE_DEBUG_ERRORS` | `false` (prod) / `true` (dev) | Detailed error pages |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | HTTP cache headers |
| `RUNIQUE_ALLOWED_HOSTS` | `*` | Allowed hosts for the host validation middleware |

> **CSP** — The CSP policy is configured exclusively via the builder (`.with_csp(...)`), with no environment variable. See [CSP](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md).

---

## Rate Limiting

| Variable                         | Default | Description                                              |
|----------------------------------|---------|----------------------------------------------------------|
| `RUNIQUE_RATE_LIMIT_REQUESTS`    | `60`    | Number of requests allowed per window (`RateLimiter`)    |
| `RUNIQUE_RATE_LIMIT_WINDOW_SECS` | `60`    | Window duration in seconds (`RateLimiter`)               |

See [Rate Limiting](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/rate-limit/rate-limit.md) for usage.

---

## LoginGuard

| Variable                       | Default | Description                                                 |
|--------------------------------|---------|-------------------------------------------------------------|
| `RUNIQUE_LOGIN_MAX_ATTEMPTS`   | `5`     | Number of failures before account lockout (`LoginGuard`)    |
| `RUNIQUE_LOGIN_LOCKOUT_SECS`   | `300`   | Lockout duration in seconds (`LoginGuard`)                  |

See [LoginGuard](https://github.com/seb-alliot/runique/blob/main/docs/en/auth/login-guard/login-guard.md) for usage.

---

## Sessions

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_SESSION_CLEANUP_SECS` | `60` | Periodic cleanup interval (seconds) |
| `RUNIQUE_SESSION_LOW_WATERMARK` | `134217728` (128 MB) | Proactive cleanup threshold — background purge of expired anonymous sessions (bytes) |
| `RUNIQUE_SESSION_HIGH_WATERMARK` | `268435456` (256 MB) | Emergency threshold — synchronous cleanup + 503 if still exceeded (bytes) |

See [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md) for behavioral details.

---

## See also

| Section | Description |
| --- | --- |
| [Application & Server](https://github.com/seb-alliot/runique/blob/main/docs/en/env/application/application.md) | DEBUG, IP_SERVER, PORT, DB, Redirects |
| [Assets & media](https://github.com/seb-alliot/runique/blob/main/docs/en/env/assets/assets.md) | Static files, media, templates |

## Back to summary

- [Environment Variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)
