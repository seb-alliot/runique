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
| `DATABASE_URL` | — | Full URL. **Read first**: when set, `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT` and `DB_NAME` are ignored, and the engine comes from the URL's scheme (`postgres://`, `mysql://`, `mariadb://`, `sqlite://`) |
| `DB_ENGINE` | `sqlite` | Engine, when `DATABASE_URL` is unset: `postgres`, `mysql`, `mariadb`, `sqlite`. Must match the compiled `runique` feature, or startup fails with "driver not enabled" |
| `DB_USER` | — | Username (required except for SQLite) |
| `DB_PASSWORD` | — | Password (required except for SQLite) |
| `DB_HOST` | `localhost` | Host |
| `DB_PORT` | `5432` / `3306` | Port (default depends on engine) |
| `DB_NAME` | `local_base.sqlite` (SQLite only) | Database name — **required** for postgres/mysql/mariadb, startup fails if absent |

### Connection pool

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_MAX_CONNECTIONS` | `100` | Maximum pool size |
| `DB_MIN_CONNECTIONS` | `20` | Minimum pool size |

### Timeouts

| Variable | Default | Unit | Description |
|----------|---------|------|-------------|
| `DB_CONNECT_TIMEOUT` | `2` | seconds | Connection establishment timeout |
| `DB_ACQUIRE_TIMEOUT` | `500` | milliseconds | Pool acquire timeout |
| `DB_IDLE_TIMEOUT` | `300` | seconds | Idle connection lifetime |
| `DB_MAX_LIFETIME` | `3600` | seconds | Maximum connection lifetime |

### SQL Logging

| Variable | Default | Description |
|----------|---------|-------------|
| `DB_LOGGING` | `false` | Enable SQL query logging (`true`, `1`, `yes`) |

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
