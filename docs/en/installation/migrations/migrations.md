# Migrations

## The `migration` crate

Migrations live in a Rust crate of their own, `migration/`, part of the project's workspace:

```
migration/
├── Cargo.toml        runique + sea-orm-migration, same engine as the app
└── src/
    ├── main.rs       runs SeaORM's CLI (up, down, status)
    ├── lib.rs        the list of migrations, kept up to date by makemigrations
    ├── m<date>_*.rs  the generated migrations
    └── snapshots/    the schema already migrated, model by model
```

**You don't create it**: `runique new` writes it with the project, and `runique makemigrations` creates it when it's missing (a project not created by `runique new`), with the engine of `DB_ENGINE`. There is nothing to write by hand in `lib.rs`.

### The expected `Cargo.toml`

Runique writes it once, at creation; after that it's yours and Runique never touches it again. Here is the file as generated (Postgres example):

```toml
[package]
name = "migration"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "migration"
path = "src/lib.rs"

[dependencies]
runique = { version = "4.0.0", features = ["postgres"] }
sea-orm-migration = { version = "=2.0.4", features = [
    "runtime-tokio-rustls",
    "sqlx-postgres",
] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
async-trait = "0.1"
```

What is required:

- **`runique`** with the same engine feature as the app (`sqlite`, `postgres`, `mysql` or `mariadb`): it provides the framework's tables.
- **`sea-orm-migration`** at the **exact same version** as Runique's, with `runtime-tokio-rustls` and the engine's driver: `sqlx-sqlite`, `sqlx-postgres`, or `sqlx-mysql` (MySQL and MariaDB alike).
- **`sea-orm-migration`'s default features**: they bring `cli::run_cli`, called by `src/main.rs`. With `default-features = false`, add `"cli"` to the list, or the crate no longer compiles.
- **`tokio`** with `macros` and `rt-multi-thread` (`#[tokio::main]` in `main.rs`), and **`async-trait`** (the generated migrations).
- **`[lib] name = "migration"`**: `main.rs` calls `migration::Migrator`.

---

## Two-step workflow

### 1. Write the migration files

```bash
runique makemigrations
```

`makemigrations` reads your models in `src/entities/`, compares them to the snapshots, and writes what changed:

| Pass | Source | Result |
| --- | --- | --- |
| 1 — App tables | `model!{}` blocks | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Framework extensions | `extend!{}` blocks | `ALTER TABLE ADD / DROP COLUMN` on an `eihwaz_*` table |
| 3 — Framework tables | Automatic | `eihwaz_*` migrations placed at the top of `lib.rs` |

The command ends with the number of files written, or "No changes detected" when nothing changed, then gives the next command.

### 2. Apply the migrations

```bash
runique migration up
```

### Roll back or check

```bash
runique migration down -n 1    # rolls back the last migration (its down() runs)
runique migration status       # lists the migrations and their state
```

These commands run the `migration` crate (`cargo run`), which reads `DATABASE_URL` from `.env`: run them from the project root. `sea-orm-cli` isn't needed, and the SeaORM version used is the one `migration/Cargo.toml` pins, the same as runique's.

> `runique makemigrations` is the only tool to use to **write** migrations. Don't use `sea-orm-cli migrate generate`: Runique keeps its own snapshots and the order of `lib.rs`.

---

## Destructive changes — `--force`

Before writing anything, `makemigrations` spots destructive changes and **refuses** to write the migration, unless `--force` is given:

- `DROP COLUMN` (data loss)
- a column's type changing (see the limits below)
- `nullable` → required (needs a default value or a prior backfill)
- a shorter length (smaller `max_length`): longer values would be truncated or refused
- a dropped foreign key (orphaned rows possible)
- an `ON DELETE CASCADE` FK added over existing data

```bash
runique makemigrations --force
```

The check covers the app's tables (`model!{}`) as well as the extensions of the framework's tables (`extend!{}`).

---

## Foreign keys and target engine

The generated files are **shared by every engine**: what concerns one engine only is chosen at run time, in the file.

- FKs are declared **in the `CREATE TABLE`**, and referenced tables are created before the tables referencing them.
- **Cycle** (two new tables referencing each other): under SQLite, the key stays in the `CREATE TABLE`; under PostgreSQL / MySQL / MariaDB, the key closing the cycle is added by an `ALTER TABLE` once its target table exists.
- Each `belongs_to` column gets an `idx_<table>_<column>` index.
- Enums become a `CREATE TYPE … AS ENUM` under PostgreSQL, a `VARCHAR` elsewhere.

Switching engines therefore needs no new migrations.

---

## Framework tables — provided automatically

Five framework migrations are placed at the top of `lib.rs`, with nothing to declare:

| Migration | Creates | Order |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_superuser, is_dirigeant, is_admin, session_version, activated_at, created_at, updated_at) | 1st |
| `EihwazSessionsMigration` | `eihwaz_sessions` (persisted authenticated sessions) | 2nd |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes` | 3rd |
| `EihwazHistoryMigration` | `eihwaz_history` (the admin's change history) | 4th |
| `EihwazResetTokensMigration` | `eihwaz_reset_tokens` (hashed reset tokens, one live per account) | 5th |

> Don't declare `eihwaz_users` among your entities: to add columns to it, use `extend!{}`.

---

## Extending a framework table — `extend!{}`

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        avatar: image [nullable, upload_to: "avatars/"],
        bio: textarea [nullable],
        website: url [nullable],
    }
}
```

`makemigrations` writes the matching `ALTER TABLE ADD COLUMN`. The extension's snapshot is stored in `migration/src/snapshots/runique/<table>.rs`.

**Extensible tables:** `eihwaz_users` and `eihwaz_groupes`, and only those. Extending any other table is a **compile** error.

---

## Atomic generation

The changes of the models and of the `extend!{}` blocks are first planned in memory, checked (destructive check, identifier lengths), then written in one batch. On a write error, the files already written are undone, and the snapshots as well as `lib.rs` are restored.

---

## Current limits

- **A column's type change isn't migrated yet.** `makemigrations` blocks it; with `--force`, the migration it writes holds only a warning comment, no `ALTER`, while the snapshot already records the new type. The console says so in yellow. Before `runique migration up`, write that migration's `up()` and `down()` by hand (SeaORM's `alter_table` / `modify_column`), or add a new column, copy the data over, then drop the old one. A change to or from an enum is migrated.
- **Dropping a table isn't detected.** Removing a model generates nothing: `makemigrations` answers "no changes", and the table stays in the database. To drop it, write a migration by hand (`drop_table`, registered in `lib.rs`), remove its snapshot from `migration/src/snapshots/`, and first remove any `belongs_to` pointing at it. Planned for 4.1.
- **Generation isn't protected yet against a hard stop** (power cut, `kill -9`) in the middle of writing: an ordinary error is undone, an interruption may leave files partly updated. `migration/` being versioned, `git` restores it.
- **Don't copy another project's snapshots without its migrations**: a snapshot means "already migrated". `makemigrations` then writes nothing for those models, while no migration creates their tables.
- **SQL identifiers are capped at 64 characters under MySQL / MariaDB** (index and foreign key names built from table and column names): `makemigrations` refuses an identifier that is too long rather than truncate it; shorten the names involved.
- **`runique migration up` applies a migration atomically under Postgres only.** sea-orm-migration wraps each migration (and its row in `seaql_migrations`) in a transaction on that engine alone. Under SQLite and MySQL / MariaDB, a migration failing midway can stay half applied; under MySQL / MariaDB no transaction would help (each DDL statement commits on its own). In development under SQLite, delete the database file and run `runique migration up` again; elsewhere, fix the database by hand before running it again.

---

## See also

| Section | Description |
| --- | --- |
| [Database](/docs/en/installation/database) | SQLite, PostgreSQL, MySQL / MariaDB |
| [Runique CLI](/docs/en/installation/cli) | Available commands |
| [Models](/docs/en/model) | The `model!{}` DSL |
| [How makemigrations works](/docs/en/architecture/makemigrations) | Snapshots, diff, generation |

## Back to summary

- [Installation](/docs/en/installation)
