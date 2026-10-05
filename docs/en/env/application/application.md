# Application, Server & Database

## Application

| Variable | Default | Description |
|----------|---------|-------------|
| `DEBUG` | `false` | Global dev/prod switch — read **once** at startup via `LazyLock`. Enables: `debug` log level, detailed error pages, admin template hot reload. In production (`false`): `warn` level, generic errors. |
| `BASE_DIR` | `.` | Application root directory |
| `TZ` | `UTC` | IANA timezone for the application (e.g. `Europe/Paris`, `America/New_York`). Accessible via `config.timezone` — parse it with `chrono-tz` in your project. |
| `LANG` | system locale | CLI language (`fr`, `en`, `de`, `es`, `it`, `pt`, `ja`, `zh`, `ru`). Priority: `.env` > system locale (`LC_ALL`, `LC_MESSAGES`) > `en` |

---

## Server

| Variable | Default | Description |
|----------|---------|-------------|
| `IP_SERVER` | `127.0.0.1` | Listening IP address |
| `PORT` | `3000` | Listening port |
| `SECRET_KEY` | `default_secret_key` | Secret key (CSRF, signatures). In production (`DEBUG=false`), **boot fails** if it's empty, equal to the default, or under 32 characters |

---

## Database

### Connection

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | — | Full connection URL (takes priority over all component variables) |
| `DB_ENGINE` | `sqlite` | Engine: `postgres`, `mysql`, `mariadb`, `sqlite` |
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

```rust
// main.rs
let redis = redis::Client::open("redis://127.0.0.1/")?;
let db = DatabaseConfig::from_env()?.build().connect().await?;

RuniqueAppBuilder::new(config)
    .with_database(db)
    .with_custom_db(redis)   // T: Any + Send + Sync + 'static
    .routes(url::urlpatterns())
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
| [Security & sessions](/docs/en/env/security) | ALLOWED_HOSTS, CSP, Middlewares, Sessions |

## Back to summary

- [Environment Variables](/docs/en/env)
