# Known limits

What Runique 4.0 doesn't do yet, or does with a restriction. Each line points to the page explaining the limit and what to do meanwhile. Open security findings are kept apart, in `faille.md` at the repository root.

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
| Several forms on one page: `request.method.is_safe()` doesn't tell which one was sent (the global message would show under all of them) | [RuniqueForm trait](/docs/en/formulaire/trait) |
| A failed session write loses the flash message silently | [Flash macros](/docs/en/flash/macros) |
| A published file followed by a hard stop before the database commit can leave an orphan file | [Fields](/docs/en/formulaire/fields) |

## Security, sessions, authentication

| Limit | Details |
| --- | --- |
| `RateLimiter` and `LoginGuard` have no memory cap<br>`RateLimiter` and `LoginGuard` count per process: with several instances, the quota is multiplied | [Rate limiting](/docs/en/middleware/rate-limit) |
| `LoginGuard` locks by username: someone can lock an account whose name they know | [LoginGuard](/docs/en/auth/login-guard) |
| The in-memory session store belongs to one process | [Session store](/docs/en/session/store) |
| A custom store (`with_session_store`) can't be walked per user: a password reset doesn't close its sessions | [Session](/docs/en/auth/session) |
| `Delegated` mode: no external provider (OAuth, LDAP, SAML) is implemented | [Passwords](/docs/en/configuration/password) |
| ACME: one site per machine (port 80 exclusive) | [Network](/docs/en/installation/network) |

## Admin

| Limit | Details |
| --- | --- |
| `.links`, `.search_help`, `.empty_value` accepted but with no effect on the display | [ModelAdmin builder](/docs/en/admin/declaration-macro) |
| No global theme stylesheet (workaround through a block)<br>`.with_htmx` is global, no per-model list fragment<br>A missing page template or stylesheet is only reported when shown | [Template override](/docs/en/admin/template-surcharge) |
| No session management interface | — |
| No many-to-many in the DSL nor in the admin | [Roadmap](/docs/en/admin/evolution) |

## Local development

| Limit | Details |
| --- | --- |
| Cookies ignore the port: two projects on `127.0.0.1` share (and overwrite) the session cookie | [Troubleshooting](/docs/en/installation/troubleshooting) |

---

## Back to summary

- [Summary](https://github.com/seb-alliot/runique/blob/main/README.md)
