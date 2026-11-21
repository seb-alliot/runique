# Makemigrations Internals

The `runique makemigrations` command bridges your Rust entities (`model!{}`, `extend!{}`) and the database schema. It compares your declarations with the last recorded state and writes the SeaORM migrations that take the database from one to the other.

---

## The generation pipeline

### Pass 1: reading the DSL

`makemigrations` reads the `src/entities/*.rs` files without compiling them, with **the same parser as the `model!{}` macro**: the `runique_dsl` crate. A model the macro refuses is refused by the CLI, at the same place and with the same message:

- an error (unknown type or attribute, unknown foreign key action, nullability rule broken…) stops the command with the **file, line and column**;
- a file that isn't valid Rust, or that declares two `model!{}`, is an error — never a file silently skipped;
- **only attributes decide**: a column's name (`created_at`, `cache_key`…) has no effect.

`belongs_to` targets are then resolved the way the macro does: by module name (the `target.rs` file) or by framework table. The foreign key references the target's real table and real primary key; an unresolved target is an error. Every `belongs_to` column gets an index `idx_<table>_<column>` (unless it is already `unique` or leads an index of `meta`), and every list field (`multichoice` / `checkbox`) its table `{table}_{field}`.

### Pass 2: diff and snapshots

Runique keeps in `migration/src/snapshots/` the state of each table **as it was applied**, as a migration file read back on every run. That file is written by `runique migration up`, once the migration is applied; `makemigrations` never writes it. The diff detects:

- added tables, and added or dropped columns. A table whose model is gone isn't detected (see [the limits](/docs/en/installation/migrations));
- **column renames** through `[renamed_from: "old"]`: a `RENAME COLUMN` instead of a `DROP` + `ADD` (without that hint, the non-interactive tool can't guess the intent);
- changes of type, nullability, uniqueness, default value and **length** (`max_length`);
- added, dropped or renamed enum values (a rename is **one** operation);
- added or dropped foreign keys and indexes.

A snapshot written before 3.0 has no column lengths: it takes the model's once, without producing a migration, and is then rewritten with them.

### Pass 3: generation

The diff becomes SeaQuery statements (`Table::create()`, `Table::alter()`, `Index::create()`…) written to new files `migration/src/m<timestamp>_*.rs`, registered in the `Migrator` of `lib.rs`.

1. **Order**: new tables are sorted so a referenced table is created before the tables referencing it; the order is deterministic.
2. **Foreign keys**: declared in the `CREATE TABLE`. For a **cycle** of new tables referencing each other, the key that closes the cycle stays in the `CREATE TABLE` on SQLite (which only checks a key when rows are written), and is added by an `ALTER TABLE` on Postgres and MySQL, once the target table exists.
3. **Framework tables**: `eihwaz_users`, sessions, the admin tables and reset tokens have their migrations in Runique; `makemigrations` puts them first in `lib.rs`. An `eihwaz_*` table declared in `src/entities/` is never recreated — extend it with `extend!{}`.

### One file, every engine

A migration file is written once and must apply on any engine. What only concerns one engine is therefore **chosen at run time**, inside the file, rather than at generation:

- **Enums**: `CREATE TYPE … AS ENUM` on Postgres only; `VARCHAR` elsewhere. A value rename becomes `ALTER TYPE … RENAME VALUE` on Postgres and an `UPDATE` of the data elsewhere.
- **Keys of a cycle**: see above.
- **`auto_now` / `auto_now_update`**: no trigger. The entity fills these columns itself (`ActiveModelBehavior::before_save`), the same way on every engine; the migration only adds `DEFAULT CURRENT_TIMESTAMP`.
- **Column modification on SQLite**: SQLite can't modify an existing column; the file says so in a comment and applies the change on the other engines.

---

## Atomic commit & destructive guard

The passes above only *compute* an in-memory plan — nothing is written until the full plan (`model!{}` changes plus `extend!{}` changes) is assembled and validated:

1. **Destructive guard**: blocked unless `makemigrations --force` is passed:
    - `DROP COLUMN`;
    - column type change;
    - `nullable → not null`;
    - **length shrink** (smaller `max_length`, or a length added to a column that had none);
    - foreign key removal;
    - adding an `ON DELETE CASCADE` key on an existing table.

   The check covers both `model!{}` and `extend!{}` changes.

   > **Exception — column type change**: `--force` unblocks the command, but **never** generates the real `ALTER` for a type change (`String → Decimal`, etc.). The generated file only contains a `// Manual migration required.` comment — write it yourself. The console says so in yellow after the generation, column by column. The other destructive categories generate the real SQL as soon as `--force` is passed. This exception is deliberate: a generic type conversion has no reliable cross-engine cast rule (Postgres needs an explicit `USING`, MySQL converts according to its SQL mode, SQLite doesn't support `ALTER COLUMN TYPE` at all).
2. **Protected writing**: the migrations, `lib.rs` and the folders created for them are written as one batch, together with the removal of the pending migrations being replaced. On a write error, everything is undone: the generated files are removed, the replaced migrations and `lib.rs` go back to their previous state. A hard stop (power cut, `kill -9`) is repaired by the next run, from the journal left in `migration/src/`. See [Atomic file writes](/docs/en/architecture/atomic).

---

## Why customized snapshots?

`makemigrations` compares your models to **the last state it generated itself**, recorded in the snapshots. The result is therefore the same on every machine of the team, and doesn't depend on a database that may have been changed by hand.

Since 4.1, the command **connects to the database first** (the one of the `.env`, as the app does) and stops with an error when it doesn't answer, before anything is read or written. It writes nothing there: only `runique migration up` applies a migration.

It reads there the list of applied migrations (`seaql_migrations`): when one of them is unknown to `lib.rs`, the command stops before anything is computed. A database that applied nothing yet (no such table) is a normal case.

## Running again before applying

Until `runique migration up` has run, the migrations written are **pending** (they carry the line `// runique: waiting migrate up`) and their tables have no snapshot. Running `makemigrations` again therefore starts from the same snapshots and computes the whole gap to the models again:

- **nothing changed**: the result is the same migrations, no file is rewritten;
- **a model changed**: the pending migrations are removed and replaced by the new result. They don't pile up: a table created then changed before being applied gives a single, complete creation;
- **the change was taken back** (the models match the snapshots again): the pending migrations are removed, with their line in `lib.rs`;
- **the run is refused** (a destructive change without `--force`): nothing is written, what was pending stays as it was.

A migration already applied is never replaced.

> A pending migration edited by hand is overwritten by the next run. If a migration must be written by hand, apply it before running `makemigrations` again.

The trade-off: a snapshot means "already migrated". Snapshots copied from another project without their migration files make `makemigrations` believe the tables exist; it then writes nothing for them.

### `extend!{}` logic

When you use `extend! { table: "eihwaz_users", ... }`, `makemigrations`:
1. Identifies the targeted framework table.
2. Stores the extension in a dedicated snapshot folder.
3. Generates an `ALTER TABLE` instead of a `CREATE TABLE`.

---

## Concrete examples

### Renaming a column without data loss

Renaming a field directly produces a `DROP` + `ADD` → data lost. The `renamed_from` hint tells the non-interactive tool what you mean:

```rust
model! {
    Employee,
    table: "employees",
    pk: id => i32,
    {
        // before:  job_title: text,
        title: text [renamed_from: "job_title"],
    }
}
```

`makemigrations` then emits `ALTER TABLE employees RENAME COLUMN job_title TO title` (PostgreSQL, MySQL, SQLite). The attribute is a migration-only directive: no effect on the generated entity or form. Safeguard: if the old column still exists in the snapshot (stale hint), no rename is emitted.

### Extending a framework table with `extend!{}`

To add columns to `eihwaz_users` (or `eihwaz_groupes`) without touching the framework:

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio:         textarea [nullable],
        avatar:      image    [nullable, upload_to: "avatars/"],
        website:     url      [nullable],
        is_verified: bool     [default: false],
    }
}
```

On the next `makemigrations`, these fields become an `ALTER TABLE eihwaz_users ADD COLUMN …` (never a `CREATE TABLE`). `extend!{}` fields follow the same types, options and nullability rules as `model!{}`, `renamed_from` included.

### Generating, applying, rolling back

```bash
# Detect the diff and write the migration files
runique makemigrations

# Destructive changes are blocked by default. To allow them:
runique makemigrations --force
# NB: for a column type change, --force only unblocks the run — the
# generated file still just has a "Manual migration required." comment,
# never a real ALTER (see the note above).

# Custom paths (defaults: src/entities and migration/src)
runique makemigrations --entities src/entities --migrations migration/src

# Apply the generated migrations
runique migration up

# Roll back the last N / see the state
runique migration down -n 1
runique migration status
```

---

← [**Architecture**](/docs/en/architecture) | [**Models**](/docs/en/model) →
