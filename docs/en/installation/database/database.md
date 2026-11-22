# Database

## Choosing the engine (Cargo features)

| Feature | What it brings |
|---|---|
| `orm` (default) | SeaORM: entities, queries, migrations, sessions. **No driver**: the code compiles and `makemigrations` works, but the app can't connect to any database. |
| `sqlite`, `postgres`, `mysql` | The engine's driver. Each one also turns `orm` on. |
| `all-databases` | All three drivers at once, for multi-engine tools (CLI, CI). |

An app turns on **one** engine: turning on two is a compile error (except through `all-databases`). `orm` needn't be written.

MariaDB is not supported: SeaORM only knows these three engines, and refuses a `mariadb://` URL.

---

## SeaORM versions

Runique pins an **exact** SeaORM version. The `Cargo.toml` files written by `runique new` carry it automatically.

| Runique | SeaORM / sea-orm-migration |
|---|---|
| 4.1.0-preview | `=2.0.4` |
| 4.0.0 | `=2.0.4` |

Don't change that version by hand: when you upgrade Runique, align `Cargo.toml` and `migration/Cargo.toml` on the version the new release uses. A different version in the project is refused by Cargo when it resolves the dependencies.

---

## Switching engines: three places to change together

A project created by `runique new` uses SQLite. To move to another engine, change **all three**:

**1. `Cargo.toml`** — runique's feature:

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
```

**2. `migration/Cargo.toml`** — runique's feature **and** the `sea-orm-migration` driver:

```toml
[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
```

| Engine | `runique` feature | `sea-orm-migration` feature |
|---|---|---|
| SQLite | `sqlite` | `sqlx-sqlite` |
| PostgreSQL | `postgres` | `sqlx-postgres` |
| MySQL | `mysql` | `sqlx-mysql` |

**3. `.env`** — `DB_ENGINE` and `DATABASE_URL`:

```env
DB_ENGINE=postgres
DATABASE_URL=postgres://runique_user:secure_password@localhost:5432/my_project
```

When `.env` names an engine that isn't compiled, the app refuses to start with "driver not enabled"; the message names the compiled engine, so you can align one or the other.

Then apply the migrations to the new database: `runique migration up`.

### `DATABASE_URL` examples

```env
DATABASE_URL="sqlite://data.db?mode=rwc"                        # file created if missing
DATABASE_URL=postgres://user:password@localhost:5432/my_db
DATABASE_URL=mysql://user:password@localhost:3306/my_db
```

A relative SQLite path is read from the folder the app runs in: run `cargo run` and `runique migration` from the project root.

---

## How Runique picks the connection

The same rule serves the app, `runique makemigrations` and `runique migration up | down | status`. Two sources are read from the `.env`:

| Source | Content |
| --- | --- |
| `DATABASE_URL` | the full address, for example `postgres://app:secret@db.local:5432/shop` |
| separate keys | `DB_ENGINE`, `DB_USER`, `DB_PASSWORD`, `DB_HOST`, `DB_PORT`, `DB_NAME` |

Runique imposes neither. `DATABASE_URL` is read first; the keys take over when it is absent or incomplete, and stand by when it is complete.

### The table

| `DATABASE_URL` | Separate keys | What Runique does |
| --- | --- | --- |
| complete | whatever | uses the address. When the connection fails, see "When the address doesn't answer" |
| incomplete | enough | warns on the console, connects with the keys, then reminds that `DATABASE_URL` is incomplete |
| incomplete | partial | refuses to start, saying why the address is incomplete and what the keys lack |
| incomplete, towards a server or with no known engine | none | refuses to start: SQLite is never taken instead |
| `sqlite://` without a file name | none | warns, then uses the file `local_base.sqlite` |
| absent | enough | builds the address from the keys |
| absent | partial | refuses to start, saying what is missing |
| absent | none | SQLite, file `local_base.sqlite` |

A key that is set but has no value (`DB_USER=`) counts as absent, `DATABASE_URL` included. A commented line is never read: that is the normal way to turn a source off.

### When an address is incomplete

In four cases only:

| Case | Example |
| --- | --- |
| its start names no supported engine | `mariadb://…`, `app:secret@db.local/shop` |
| it can't be read | a port that is not a number (`:543a`), a password holding an unencoded `/`, `#` or `?` |
| it aims at a server without naming a database | `postgres://app:secret@db.local`, `postgres://app:secret@db.local/` |
| it asks for SQLite without naming a file | `sqlite://`, `sqlite:` |

Anything else is tried as it is, and the server decides. On PostgreSQL, `?dbname=shop` counts as a database name.

### What Runique leaves to the driver

For an address missing a part that is not the database, Runique follows SeaORM and its driver, adding nothing:

| Missing part | PostgreSQL | MySQL |
| --- | --- | --- |
| user | the machine's `PGUSER`, otherwise the system account name | `root` |
| password | the machine's `PGPASSWORD`, otherwise none | none |
| host | the machine's `PGHOST`, otherwise `localhost` | `localhost` |
| port | the machine's `PGPORT`, otherwise 5432 | 3306 |

An address without a password (`postgres://app@db.local/shop`) or without a user (`postgres://db.local/shop`) is therefore tried: some servers accept it. For PostgreSQL, the missing parts are filled in from the machine's `PG*` variables, not from the `DB_*` keys of the `.env`.

Other accepted forms: `sqlite:data.db`, `sqlite:///absolute/path.db`, `sqlite::memory:`; an IPv6 host (`postgres://[::1]/shop`); a socket connection (`?host=/var/run/postgresql`). An unknown parameter after the `?` is left to the driver.

### The separate keys

| Key | Server (PostgreSQL, MySQL) | SQLite |
| --- | --- | --- |
| `DB_ENGINE` | required: `postgres` (or `postgresql`), `mysql` | `sqlite` |
| `DB_USER` | required | ignored |
| `DB_PASSWORD` | required | ignored |
| `DB_HOST` | optional, `localhost` by default | ignored |
| `DB_PORT` | optional, the engine's own by default | ignored |
| `DB_NAME` | required: the database name | optional: the file, `local_base.sqlite` by default |

- Case and spaces don't matter in `DB_ENGINE`. Any other value is an error listing the accepted engines.
- **Keys without `DB_ENGINE` are an error**: without the engine no address can be built, and Runique doesn't take SQLite in its place.
- **The user and the password are written as they are**: Runique encodes them while building the address. A password `p@ss/w:rd#1` is given that way, with nothing encoded. In `DATABASE_URL`, encoding it is up to you (`p%40ss%2Fw%3Ard%231`), otherwise the address can't be read.
- A value holding `#` or `$` goes between single quotes in the `.env` (`DB_PASSWORD='pa$$#1'`): without them, the file reader cuts the value at `#` and replaces `$name` with another variable.

### When the address doesn't answer

A complete `DATABASE_URL` whose connection fails (server stopped, password refused, no such database) doesn't block right away: Runique warns on the console and **tries the separate keys**.

| Separate keys | What follows |
| --- | --- |
| enough | connects with the keys; a failure blocks the start |
| partial | start blocked: the message gives the address's failure and what the keys lack |
| none | start blocked on the address's failure |

The second try happens even when the keys name the same database: it shows whether the failure came from `DATABASE_URL`. It costs the connection timeout of the first one (`DB_CONNECT_TIMEOUT`, 2 s by default).

> **To check in your `.env`**: the two sources may contradict each other, and `DATABASE_URL` wins as long as it answers. A `DB_ENGINE=sqlite` left next to a server address becomes the database in use as soon as the server stops answering. In production, keep one source only, or two that name the same database.

`runique migration up | down | status` look for the address that answers the same way, then hand it to the `migration` crate, which only reads `DATABASE_URL`: the app and the migrations always aim at the same database. The password never appears in an error message.

---

## PostgreSQL

### 1. Install PostgreSQL

**macOS:**

```bash
brew install postgresql
brew services start postgresql
```

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get install postgresql postgresql-contrib
sudo systemctl start postgresql
```

**Windows:** [download the installer](https://www.postgresql.org/download/windows/) and follow the wizard.

### 2. Create the user and the database

```sql
-- As an administrator: psql -U postgres
CREATE USER runique_user WITH PASSWORD 'secure_password';
CREATE DATABASE my_project OWNER runique_user;
GRANT ALL PRIVILEGES ON DATABASE my_project TO runique_user;
GRANT ALL PRIVILEGES ON SCHEMA public TO runique_user;
```

### 3. Check the connection

```bash
psql -U runique_user -d my_project -h localhost
```

---

## Current limits

- **One engine per build.** The ORM (SeaORM) talks to a single engine at a time (except `all-databases`, meant for tools). Nothing stops you from attaching **another** external database — Redis, MongoDB, a spare PostgreSQL: a nod to [`with_custom_db`](/docs/en/configuration/builder).
- **SQLite accepts one writer at a time** over the whole database: under heavy write load, a query may wait, then fail (`SQLITE_BUSY`). It is the development engine; prefer PostgreSQL in production.
- **The generated migrations are shared by every engine**: what concerns one engine only (PostgreSQL's native `CREATE TYPE` for enums, the foreign keys of a cycle) is chosen at run time, in the file. Switching engines needs no new migrations, but a **fresh database**: data isn't carried from one engine to another.

---

## See also

| Section | Description |
| --- | --- |
| [Prerequisites](/docs/en/installation/prerequisites) | Initial setup |
| [Migrations](/docs/en/installation/migrations) | Migration workflow |

## Back to summary

- [Installation](/docs/en/installation)
