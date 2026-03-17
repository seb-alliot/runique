# Security, Middlewares, CSP & Sessions

## Middlewares

| Variable | Default | Description |
| --- | --- | --- |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | HTTP cache headers |

> **CSP** — Configured exclusively via the builder (`.with_csp(...)`). See [CSP](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/csp/csp.md).
> **Host validation** — Configured exclusively via the builder (`.with_allowed_hosts([...])`). See [Host Validation](https://github.com/seb-alliot/runique/blob/main/docs/en/middleware/hosts/hosts.md).

---

## Sessions

| Variable | Default | Description |
| --- | --- | --- |
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
