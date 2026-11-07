# Security, Middlewares, CSP & Sessions

## Middlewares

No middleware is configured through `.env`. In development (`RUNIQUE_ENV=dev`), no-cache headers are added on `localhost`; `.middleware(|m| m.with_cache(true))` turns them off.

> **CSP** — Configured exclusively via the builder (`.with_csp(...)`). See [CSP](/docs/en/middleware/csp).
> **Host validation** — Configured exclusively via the builder (`.with_allowed_hosts(|h| h.host("..."))`). See [Host Validation](/docs/en/middleware/hosts-cache).

---

## HTTPS, ACME and logging

| Variable | Default | Description |
|----------|---------|-------------|
| `ENFORCE_HTTPS` | `false` | Redirects HTTP to HTTPS behind a proxy (see [Forced HTTPS](/docs/en/middleware/csp)) |
| `HSTS_MAX_AGE` | `31536000` (1 year) | Duration of the `Strict-Transport-Security` header, in seconds |
| `HSTS_INCLUDE_SUBDOMAINS` | `true` | Adds `includeSubDomains` to the HSTS header |
| `HSTS_PRELOAD` | `false` | Adds `preload`. Browsers only accept it with `includeSubDomains` and `HSTS_MAX_AGE` ≥ 31536000; otherwise, a warning at startup. Near-irreversible once the domain is listed |
| `ACME_ENABLED` | `false` | Let's Encrypt certificates handled by Runique (`acme` feature) |
| `ACME_DOMAIN`, `ACME_EMAIL` | — | Required when `ACME_ENABLED=true` |
| `ACME_CERTS_DIR` | `./certs` | Certificate folder — an absolute path in production |
| `RUST_LOG` | — | Log filter (`tracing`), overrides the default level (`debug` with `RUNIQUE_ENV=dev`, `warn` otherwise) |
| `RUNIQUE_LOG_FILE` | — | Adds a log file, with no recompiling |

See [Network & HTTP](/docs/en/installation/network) for ACME.

---

## Sessions

Session memory limits and cleanup interval are configured via the builder — see [Sessions](/docs/en/session).

---

## See also

| Section | Description |
| --- | --- |
| [Application & Server](/docs/en/env/application) | RUNIQUE_ENV, DEBUG, IP_SERVER, PORT, DB |
| [Assets & media](/docs/en/env/assets) | Static files, media, templates |

## Back to summary

- [Environment Variables](/docs/en/env)
