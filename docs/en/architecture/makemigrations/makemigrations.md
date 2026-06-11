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
    - Modified constraints (e.g., changing `nullable` to `required`).

### Pass 3: SeaQuery Generation

The diff is converted into a sequence of `SeaQuery` statements (`TableCreate`, `TableAlter`).

1. **Ordering**: It ensures that dependencies (Foreign Keys) are handled in the correct order.
2. **Framework Tables**: It automatically injects the `eihwaz_users` and `eihwaz_groupes` migrations if they are missing or need extension via `extend!{}`.
3. **Rust Code Output**: It writes a new `.rs` file in `migration/src/` and updates the `Migrator` trait.

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

← [**Architecture**](/docs/en/architecture) | [**Models**](/docs/en/model) →
