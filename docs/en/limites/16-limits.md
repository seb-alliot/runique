# Known limits

What Runique 4.0 doesn't do yet, or does with a restriction. Each line points to the page explaining the limit and what to do meanwhile. The security policy and the fixed vulnerabilities are described in [`SECURITY.md`](https://github.com/seb-alliot/runique/blob/main/SECURITY.md).

---

## Database and migrations

| Limit | Details |
| --- | --- |
| One engine per build (`sqlite`, `postgres`, `mysql`)<br>SQLite: one writer at a time, `SQLITE_BUSY` under heavy write load<br>Switching engines needs a fresh database: data isn't carried over | [Database](/docs/en/installation/database) |
| A column type change isn't migrated (`--force` writes an empty migration, to complete by hand)<br>Snapshots copied without their migrations make the tables look created<br>SQL identifiers capped at 64 characters on MySQL / MariaDB<br>`migration up`: a migration is atomic under Postgres only (SQLite, MySQL / MariaDB: a failing migration can stay half applied)<br>A hard stop during `makemigrations` can leave half-written files (`git` to restore) | [Migrations](/docs/en/installation/migrations), [Makemigrations](/docs/en/architecture/makemigrations) |
| `search!`: `in (vec)` with an empty vec, no inline conditional filter | [Queries](/docs/en/orm/queries) |

## Forms and messages

| Limit | Details |
| --- | --- |
| A failed session write loses the flash message silently | [Flash macros](/docs/en/flash/macros) |
| A published file followed by a hard stop before the database commit can leave an orphan file | [Fields](/docs/en/formulaire/fields) |

## Security, sessions, authentication

| Limit | Details |
| --- | --- |
| `RateLimiter` and `LoginGuard` have no memory cap<br>`RateLimiter` and `LoginGuard` count per process: with several instances, the quota is multiplied | [Rate limiting](/docs/en/middleware/rate-limit) |
| `LoginGuard` locks by username: someone can lock an account whose name they know | [LoginGuard](/docs/en/auth/login-guard) |
| The in-memory session store belongs to one process | [Session store](/docs/en/session/store) |
| A custom store (`with_session_store`) can't be walked per user: a password reset doesn't close its sessions | [Session](/docs/en/auth/session) |
| Session duration: one for every signed-in account; "remember me" (a duration per session) is not possible yet | [Sessions](/docs/en/middleware/sessions) |
| At most 5 custom middlewares (a 6th makes startup panic); they run before the session, without the signed-in account | [Middleware stack](/docs/en/architecture/middleware) |
| `Delegated` mode: no external provider (OAuth, LDAP, SAML) is implemented | [Passwords](/docs/en/configuration/password) |
| ACME: one site per machine (port 80 exclusive) | [Network](/docs/en/installation/network) |

## Admin

| Limit | Details |
| --- | --- |
| `.links`, `.search_help`, `.empty_value` accepted but with no effect on the display | [ModelAdmin builder](/docs/en/admin/declaration-macro) |
| No global theme stylesheet (workaround through a block)<br>`.with_htmx` is global, no per-model list fragment<br>A missing stylesheet isn't reported | [Template override](/docs/en/admin/template-surcharge) |
| No session management interface | — |
| Many-to-many: declared in the DSL through a pivot table, not managed in the admin | [Roadmap](/docs/en/admin/evolution) |

## Local development

| Limit | Details |
| --- | --- |
| Cookies ignore the port: two projects on `127.0.0.1` share (and overwrite) the session cookie | [Troubleshooting](/docs/en/installation/troubleshooting) |

---

## Solutions in the meantime

| Limit | Solution |
| --- | --- |
| `RateLimiter` and `LoginGuard` have no memory cap, counters per process | Also limit at the reverse proxy (example below) |
| `LoginGuard` blocks by username | Pair it with a limit per IP, at the reverse proxy or with `RateLimiter` |
| The memory session store belongs to one process | One instance, or a reverse proxy sending each visitor to the same one (`ip_hash` with nginx) |
| ACME: one site per machine | Hand TLS to a reverse proxy as soon as there are several sites |
| SQLite: one writer at a time | PostgreSQL for a sustained write load |
| A migration is atomic only on PostgreSQL | Back up the database before `migration up` on SQLite and MySQL / MariaDB |
| A column's type change isn't migrated | Complete by hand the migration written by `--force` |
| Cookies shared by two projects on `127.0.0.1` | Open one on `127.0.0.1`, the other on `localhost` |

Limit per IP with nginx, to adapt. The zone caps the memory, and a refused request gets a 503 by default:

```nginx
limit_req_zone $binary_remote_addr zone=login:10m rate=5r/m;

location /login {
    limit_req zone=login burst=5 nodelay;
    proxy_pass http://127.0.0.1:3000;
}
```

---

## Back to summary

- [Summary](https://github.com/seb-alliot/runique/blob/main/README.md)
