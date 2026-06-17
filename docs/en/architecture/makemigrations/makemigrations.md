# Makemigrations Internals

The `runique makemigrations` command is a sophisticated tool that bridges the gap between your Rust entities (`model!{}`) and the database schema. Unlike standard ORM generators, it is designed to preserve architectural intent and framework-specific extensions.

---

## The Generated Data Pipeline

The generation process follows a three-pass architecture:

### Pass 1: AST Extraction (`parse_schema_from_source`)

Runique uses a lightweight custom parser (based on `syn` and regular expressions for performance) to read your `src/entities/*.rs` files. 

- **Static Analysis**: It doesn't compile your code. It reads the source files directly to extract the structure of `model!{}` blocks.
- **Normalizer**: It converts high-level DSL types (e.g., `datetime`, `uuid`) into internal `FieldDef` structures.
- **Intelligence**: This is where the **Automatic Field Mapping** happens (mapping field names like `email` to specialized form behaviors).

### Pass 2: Diffing & Snapshotting

Runique maintains a hidden state in `migration/src/snapshots/`. 

- **Current State**: The parser builds a virtual schema of your current code.
- **Previous State**: It loads the last snapshot from the filesystem.
- **Diffing Engine**: It compares the two states to find:
    - New tables / Deleted tables.
    - Added columns / Removed columns.
    - **Column rename**: via the explicit `[renamed_from: "old"]` hint, the diff emits a `RENAME COLUMN` instead of a `DROP` + `ADD` (no data loss). Without the hint, the non-interactive tool cannot guess intent.
    - Modified constraints (e.g., changing `nullable` to `required`).
    - **Enum values**: additions, removals and renames (by position). A rename is treated as **one** operation, excluded from the add/remove lists.

### Pass 3: SeaQuery Generation

The diff is converted into a sequence of `SeaQuery` statements (`TableCreate`, `TableAlter`).

1. **Ordering**: It ensures that dependencies (Foreign Keys) are handled in the correct order (topological sort of new tables).
2. **Framework Tables**: It automatically injects the `eihwaz_users` and `eihwaz_groupes` migrations if they are missing or need extension via `extend!{}`.
3. **Rust Code Output**: It writes a new `.rs` file in `migration/src/` and updates the `Migrator` trait.

### Engine-specific generation

The target engine is detected (`DB_URL`/`DATABASE_URL`/`DB_ENGINE`) and the output is adapted:

- **Foreign keys**: grouped into a `create_relations` migration (`ALTER … ADD CONSTRAINT`) on PostgreSQL/MySQL/MariaDB; declared **inline in the `CREATE TABLE`** on SQLite (which cannot add FKs to an existing table).
- **Enums**: `CREATE TYPE … AS ENUM` on PostgreSQL; native `VARCHAR`/`ENUM` elsewhere. An enum value rename becomes `ALTER TYPE … RENAME VALUE` on PostgreSQL (atomic) and a plain data `UPDATE` on other engines.
- **`updated_at`**: PostgreSQL trigger; `ON UPDATE CURRENT_TIMESTAMP` on MySQL/MariaDB.

Generated files are therefore **engine-specific**: to switch engines, regenerate from scratch with the right `DB_ENGINE`.

---

## Atomic commit & destructive guard

The passes above only *compute* a plan in memory — nothing is written until the full plan (`model!{}` changes plus `extend!{}` changes) is assembled and validated:

1. **Destructive guard**: `DROP COLUMN`, column type changes, `nullable → not null`, dropped foreign keys and newly added `ON DELETE CASCADE` constraints are blocked unless `makemigrations --force` is passed. The guard covers both `model!{}` and `extend!{}` changes.
2. **Single commit**: directory creation, file writes, `lib.rs` registration and `AdminTableMigration` positioning all run under one rollback. On any write error, generated files are removed and pre-existing snapshots and `lib.rs` are restored to their previous content.

---

## Why customized snapshots?

Runique doesn't rely solely on the database state (which can be desynchronized). By keeping snapshots of the **DSL state**, the framework ensures that your Admin forms always match your model declarations, even if you haven't applied the migrations yet.

### `extend!{}` logic

When you use `extend! { table: "eihwaz_users", ... }`, the parser:
1. Identifies the target framework table.
2. Stores the extension in a specific snapshot folder.
3. Generates an `ALTER TABLE` instead of a `CREATE TABLE` during the next `makemigrations` run.

---

## Concrete examples

### Renaming a column without data loss

Renaming a field directly produces a `DROP` + `ADD` → lost data. The `renamed_from` hint signals intent to the non-interactive tool:

```rust
model! {
    Employe,
    table: "employes",
    fields: {
        // before:  job_title: text,
        title: text [renamed_from: "job_title"],
    }
}
```

`makemigrations` then emits `ALTER TABLE employes RENAME COLUMN job_title TO title` (PostgreSQL, MySQL/MariaDB, SQLite). The attribute is a migration-only directive: no effect on the generated entity or form. Safeguard: if the old column still exists in the snapshot (stale hint), no rename is emitted.

### Extending a framework table with `extend!{}`

To add columns to `eihwaz_users` (or `eihwaz_groupes`) without touching the framework:

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio: textarea,
        avatar: image [upload_to: "avatars/"],
        website: url,
        is_verified: bool [default: false],
    }
}
```

On the next `makemigrations`, these fields become an `ALTER TABLE eihwaz_users ADD COLUMN …` (never a `CREATE TABLE`). `extend!{}` fields accept the same types and options as `model!{}`, including `renamed_from`.

### Generating and applying

```bash
# Detect the diff and write the migration files
runique makemigrations

# Destructive changes (DROP COLUMN, nullable → not null,
# type change, FK removal) are blocked by default.
# To allow them explicitly:
runique makemigrations --force

# Custom paths (defaults: src/entities and migration/src)
runique makemigrations --entities src/entities --migrations migration/src

# Apply the generated migrations
sea-orm-cli migrate up
```

---

← [**Architecture**](/docs/en/architecture) | [**Models**](/docs/en/model) →
