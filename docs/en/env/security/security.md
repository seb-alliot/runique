# Security, Middlewares, CSP & Sessions

## Middlewares

| Variable | Default | Description |
| --- | --- | --- |
| `RUNIQUE_ENABLE_CACHE` | `true` (prod) / `false` (dev) | HTTP cache headers |

> **CSP** — Configured exclusively via the builder (`.with_csp(...)`). See [CSP](/docs/en/middleware/csp).
> **Host validation** — Configured exclusively via the builder (`.with_allowed_hosts([...])`). See [Host Validation](/docs/en/middleware/hosts).

---

## Sessions

Session memory limits and cleanup interval are configured via the builder — see [Sessions](/docs/en/session).

---

## See also

| Section | Description |
| --- | --- |
| [Application & Server](/docs/en/env/application) | DEBUG, IP_SERVER, PORT, DB, Redirects |
| [Assets & media](/docs/en/env/assets) | Static files, media, templates |

## Back to summary

- [Environment Variables](/docs/en/env)
