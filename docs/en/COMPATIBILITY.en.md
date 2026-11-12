# Compatibility

The page's **Versions** and **Tested engines** tables are generated from `Cargo.toml` and
`docker-compose.yml`. This file only holds the part written by hand.

## Defaults

What Runique does without any setting on your side. Each value can be changed, on the page given.

| Item | Default | Page |
|---|---|---|
| Template escaping (Tera) | on for `.html` and `.xml`; `{{ var }}` is always escaped, except with the `safe` filter | [Template syntax](/docs/en/template/syntax) |
| CSRF | required for every method but `GET` and `HEAD` | [CSRF](/docs/en/middleware/csrf) |
| CSP | `default-src 'none'`, `script-src 'self'`, `style-src 'self' 'unsafe-inline'` | [CSP](/docs/en/middleware/csp) |
| Anonymous session | 5 minutes of inactivity, in memory only | [Session durations](/docs/en/middleware/sessions) |
| Logged-in session | 24 hours of inactivity, also stored in the database | [Sessions](/docs/en/session) |
| `LoginGuard` | 5 attempts, then 5 minutes locked out | [LoginGuard](/docs/en/auth/login-guard) |
| `RateLimiter` | 60 requests per minute, on the routes it is attached to | [Rate limit](/docs/en/middleware/rate-limit) |
| Passwords | hashed with Argon2 | [Passwords](/docs/en/configuration/password) |
| Request size | form and JSON: 2 MB read into memory; file uploads: 100 MB (`RUNIQUE_MAX_BODY_MB`) | [Assets and media](/docs/en/env/assets) |

## Good practices

- Use **exactly** the SeaORM version pinned above in your project and in its `migration/` crate: `runique new` writes it for you.
- Generate migrations in development (`runique makemigrations`), commit them, and only run `migration up` in production.
- Test on your production engine, not only on SQLite: some behaviours (time zones, text case, foreign-key indexes) differ from one engine to another.
- On MySQL and MariaDB, back the database up before each `migration up`: a `CREATE` or an `ALTER` is never rolled back there.

## To avoid

- Mixing another SeaORM or sea-query version into the same project: the types would no longer match.
- Running `runique makemigrations` in production.
- Writing raw SQL where `search!`, `search_cond!` or SeaORM's methods do the job: the `?` placeholder breaks Postgres.
- Putting `| safe` on a value that comes from a visitor.
