# Database

## Choosing the engine (Cargo features)

| Feature | What it brings |
|---|---|
| `orm` (default) | SeaORM: entities, queries, migrations, sessions. **No driver**: the code compiles and `makemigrations` works, but the app can't connect to any database. |
| `sqlite`, `postgres`, `mysql` | The engine's driver. Each one also turns `orm` on. |
| `mariadb` | Alias of `mysql`. |
| `all-databases` | All three drivers at once, for multi-engine tools (CLI, CI). |

An app turns on **one** engine: turning on two is a compile error (except through `all-databases`). `orm` needn't be written.

---

## SeaORM versions

Runique pins an **exact** SeaORM version. The `Cargo.toml` files written by `runique new` carry it automatically.

| Runique | SeaORM / sea-orm-migration |
|---|---|
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
| MariaDB | `mariadb` | `sqlx-mysql` |

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
DATABASE_URL=mysql://user:password@localhost:3306/my_db         # MySQL and MariaDB
```

A relative SQLite path is read from the folder the app runs in: run `cargo run` and `runique migration` from the project root.

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

- **One engine per build.** An app can't talk to two different engines at once (except `all-databases`, meant for tools).
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
