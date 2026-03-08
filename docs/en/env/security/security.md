# Security, Middlewares, CSP & Sessions

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
| `RUNIQUE_ENABLE_HOST_VALIDATION` | `true` (prod) / `false` (dev) | Validate Host header against `ALLOWED_HOSTS` |
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

See [Sessions](https://github.com/seb-alliot/runique/blob/main/docs/en/session/14-sessions.md) for behavioral details.

---

## See also

| Section | Description |
| --- | --- |
| [Application & Server](https://github.com/seb-alliot/runique/blob/main/docs/en/env/application/application.md) | DEBUG, IP_SERVER, PORT, DB, Redirects |
| [Assets & media](https://github.com/seb-alliot/runique/blob/main/docs/en/env/assets/assets.md) | Static files, media, templates |

## Back to summary

- [Environment Variables](https://github.com/seb-alliot/runique/blob/main/docs/en/env/15-env.md)
