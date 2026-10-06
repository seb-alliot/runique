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

Runique keeps the last state of each table in `migration/src/snapshots/`, as a migration file read back on every run. The diff detects:

- added or dropped tables and columns;
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

   > **Exception — column type change**: `--force` unblocks the command, but **never** generates the real `ALTER` for a type change (`String → Decimal`, etc.). The generated file only contains a `// Manual migration required.` comment — write it yourself. The other destructive categories generate the real SQL as soon as `--force` is passed. This exception is deliberate: a generic type conversion has no reliable cross-engine cast rule (Postgres needs an explicit `USING`, MariaDB casts silently without error on an invalid value, SQLite doesn't support `ALTER COLUMN TYPE` at all).
2. **Single commit**: creating directories, writing files, registering in `lib.rs` and positioning the framework migrations run under one rollback. On a write error, the generated files are removed and the existing snapshots and `lib.rs` are restored to their previous state.

---

## Why customized snapshots?

Runique doesn't rely only on the database state (which can drift). By keeping snapshots of the **DSL state**, the framework guarantees your Admin forms always match your model declarations, even before the migrations are applied.

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

`makemigrations` then emits `ALTER TABLE employees RENAME COLUMN job_title TO title` (PostgreSQL, MySQL/MariaDB, SQLite). The attribute is a migration-only directive: no effect on the generated entity or form. Safeguard: if the old column still exists in the snapshot (stale hint), no rename is emitted.

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
runique migration up            # or: sea-orm-cli migrate up

# Roll back the last N / see the state
sea-orm-cli migrate down -n 1
sea-orm-cli migrate status
```

---

← [**Architecture**](/docs/en/architecture) | [**Models**](/docs/en/model) →
