# Application, Server & Database

## Application

| Variable | Default | Description |
|----------|---------|-------------|
| `RUNIQUE_ENV` | `prod` | `dev` or `prod`, whatever the case; unset or unknown means `prod` (with a warning for an unknown value). Read **once** at startup. In `dev`: session cookie without `Secure`, `debug` log level (unless `RUST_LOG`), templates read again on every request, no-cache headers on `localhost`, the default home page when `/` isn't declared, public URL set to `http://127.0.0.1:{PORT}` (`.with_public_url()` ignored), allowed hosts not validated, no `upgrade-insecure-requests`, no SRI and static files in `no-cache`, so edited CSS reloads without a restart. In `prod`: security checks at startup (`SECRET_KEY`, `with_public_url`, the reset's mailer, ACME). |
| `DEBUG` | `false` | Shows the detailed error pages and turns every log channel on (see [Tracing](/docs/en/configuration/tracing)) — read **once** at startup. Accepts `true`/`1`/`yes`/`on` and `false`/`0`/`no`/`off`, whatever the case. Also works with `RUNIQUE_ENV=prod` to look into a problem on a live site: not recommended, as every visitor then sees those pages (a warning shows at startup). |
| `BASE_DIR` | `.` | Application root directory |
| `TZ` | `UTC` | IANA timezone for the application (e.g. `Europe/Paris`, `America/New_York`). Accessible via `config.timezone` — parse it with `chrono-tz` in your project. |
| `LANG` | system locale | CLI language (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priority: `LANG` / `LC_ALL` / `LC_MESSAGES` from `.env` > system locale > `en`. The app's language is set in code (`set_lang(Lang::En)`) |

---

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Listening port |
| `SECRET_KEY` | `default_secret_key` | Secret key (CSRF, signatures, encryption of the sessions in the database: changing it signs their users out at restart). `runique new` writes a random one (64 hex characters) into the project's `.env`. To replace it: any random string of 32 characters or more (`openssl rand -hex 32`). In production (`RUNIQUE_ENV=prod`), **startup fails** when it is empty, equal to the default, or shorter than 32 characters; in development, a warning is shown when it is missing |

---

## Database

### Connection

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Full URL. **Read first**: when set, `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT` and `DB_NAME` are ignored, and the engine comes from the URL's scheme (`postgres://`, `mysql://`, `sqlite://`) |
| `DB_ENGINE` | `sqlite` | Engine, when `DATABASE_URL` is unset: `postgres`, `mysql`, `sqlite`. Must match the compiled `runique` feature, or startup fails with "driver not enabled" |
| `DB_USER` | — | Username (required except for SQLite) |
| `DB_PASSWORD` | — | Password (required except for SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (default depends on engine) |
| `DB_NAME` | `local_base.sqlite` (SQLite only) | Database name — **required** for postgres/mysql, startup fails if absent |

### Connection pool

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_MAX_CONNECTIONS` | `10` | Maximum pool size |
| `DB_MIN_CONNECTIONS` | `2` | Connections kept open when idle |

### Timeouts

| Variable | Default | Unit | Description |
|----------|---------|------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | seconds | Connection establishment timeout |
| `DB_ACQUIRE_TIMEOUT` | `5000` | milliseconds | Pool acquire timeout |
| `DB_IDLE_TIMEOUT` | `300` | seconds | Idle connection lifetime |
| `DB_MAX_LIFETIME` | `3600` | seconds | Maximum connection lifetime |

### Tuning the pool

The pool is the set of connections the application keeps to the database. An HTTP request that reads or writes borrows one for the time of its SQL query, then gives it back. These settings apply to PostgreSQL and MySQL; with SQLite, Runique opens a single connection and ignores them.

- **`DB_MIN_CONNECTIONS`**: the connections kept open even when nobody visits the site. It is not a limit. The higher the value, the less a request waits for a connection to open after a quiet period; each open connection uses memory on the database server.
- **`DB_MAX_CONNECTIONS`**: how many SQL queries the application can run at the same time. Beyond it, requests wait for a connection to be freed.
- **`DB_ACQUIRE_TIMEOUT`**: how long a request waits for a free connection. Past it, the request fails (`PoolTimedOut` error, 500 error page). It also applies at startup, while the first connection opens.

The defaults are small on purpose: a project starts without any setting, and one site cannot hold the whole database server. They suit a site with moderate traffic.

**When to change them**

| Situation | Setting |
|---|---|
| `PoolTimedOut` errors at peak hours, slow pages while the database answers fast | raise `DB_MAX_CONNECTIONS` |
| The first requests after a quiet period are slow | raise `DB_MIN_CONNECTIONS` |
| You prefer a quick error to a page that waits | lower `DB_ACQUIRE_TIMEOUT`, never under the time a connection takes to open (`PoolTimedOut` at startup otherwise) |

**The database server's own limit**

The server has its own ceiling, for every application together, set by `max_connections`:

| Engine | Default `max_connections` | Available to an application |
|---|---|---|
| PostgreSQL | typically 100 (less if the system cannot support it, decided at install time) | 3 fewer: `superuser_reserved_connections` keeps 3 connections for superusers |
| MySQL | 151 | 151; one extra connection is kept for administration accounts |

The sum of the `DB_MAX_CONNECTIONS` of every application connecting to it (each instance counts, as do administration tools and backups) must stay under it.

When several sites share a server, give each its own account and limit it on the database side, so one site cannot take the others' connections:

```sql
-- PostgreSQL (the limit does not apply to a superuser)
ALTER ROLE my_site CONNECTION LIMIT 15;

-- MySQL
ALTER USER 'my_site'@'localhost' WITH MAX_USER_CONNECTIONS 15;
```

### SQL Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_LOGGING` | `false` | Enable SQL query logging (`true`, `1`, `yes`) |

---

## Email sending

The mailer is configured here, and nowhere else: nothing to declare in the builder. See [Mailer](/docs/en/mailer).

| Variable | Default | Description |
|----------|---------|-------------|
| `EMAIL_BACKEND` | `smtp` | `smtp` (real sending) or `console` (prints the email to the terminal, for development) |
| `SMTP_HOST` | — | SMTP server — **required** with `smtp` |
| `SMTP_USER` | — | SMTP username — **required** with `smtp` |
| `SMTP_PASS` | — | SMTP password — **required** with `smtp` |
| `SMTP_PORT` | `587` | Server port |
| `SMTP_FROM` | `SMTP_USER` (`noreply@localhost` with `console`) | Sender address |
| `SMTP_STARTTLS` | `true` | Upgrades the connection with STARTTLS |

The configuration must be complete: with `smtp`, if one of the three required variables is missing, there is no mailer. The application still starts; a send returns an error, and the admin refuses to create an account.

---

## Secondary connections — `with_custom_db`

To attach an additional database connection (Redis pool, secondary PostgreSQL, MongoDB client, etc.), use `.with_custom_db()` on the builder. The value is stored in a `HashMap<TypeId, Arc<dyn Any>>` internal to `RuniqueEngine` — **not** injected as an Axum `Extension`. Access it in handlers via `engine.custom_db::<T>()` (or its alias `engine.extension::<T>()`), which returns `Option<Arc<T>>`.

> **`with_custom_db` does not replace the ORM's database.** The SeaORM connection — entities, `search!`, `save`, the admin and migrations — stays the one from `with_database` / `with_database_config`. Values passed to `with_custom_db` are reachable **only manually** via `engine.custom_db::<T>()`; the ORM never uses them. To change the ORM engine, see [Database](/docs/en/installation/database).

```rust
// main.rs
let redis = redis::Client::open("redis://127.0.0.1/")?;
let db = DatabaseConfig::from_env()?.build().connect().await?;

RuniqueApp::builder(config)
    .with_database(db)
    .with_custom_db(redis)   // T: Any + Send + Sync + 'static
    .routes(url::routes())
    .build().await?
    .run().await
```

```rust
// handler
use redis::Client;

pub async fn my_handler(mut req: Request) -> AppResult<Response> {
    let redis = req.engine.custom_db::<Client>().expect("redis not configured");
    let mut conn = redis.get_async_connection().await?;
    // ...
}
```

Any type implementing `Any + Send + Sync + 'static` is accepted. Multiple secondary connections of different types can be registered with repeated `.with_custom_db()` calls.

---

## See also

| Section | Description |
| --- | --- |
| [Assets & media](/docs/en/env/assets) | Static files, media, templates |
| [Security & sessions](/docs/en/env/security) | Middlewares, CSP, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
