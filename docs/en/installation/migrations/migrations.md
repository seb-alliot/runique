# Migrations (SeaORM)

## Two-Step Workflow

### 1. Generate Migration Files

`runique makemigrations` reads your entities declared in `src/entities` and generates the corresponding migration files:

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

**What `makemigrations` does in a single command:**

| Pass | Source | Output |
| --- | --- | --- |
| 1 — App tables | `model!{}` blocks in `src/entities` | `CREATE TABLE` / `ALTER TABLE` |
| 2 — Framework extensions | `extend!{}` blocks in `src/entities` | `ALTER TABLE ADD/DROP COLUMN` |
| 3 — Admin positioning | `RUNIQUE_USER_TABLE` in `.env` | Guaranteed order in `lib.rs` |

### 2. Apply Migrations

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

---

## Framework Tables — Provided Automatically

Runique automatically injects two migrations into your `lib.rs` without you having to define them:

| Migration | Creates | Order |
| --- | --- | --- |
| `EihwazUsersMigration` | `eihwaz_users` (id, username, email, password, is_active, is_staff, is_superuser, created_at, updated_at) | 1st |
| `AdminTableMigration` | `eihwaz_groupes`, `eihwaz_groupes_droits`, `eihwaz_users_groupes`, `eihwaz_history` | 2nd |

> You do not need to declare `eihwaz_users` in your entities.

---

## Extending a Framework Table — `extend!{}`

To add columns to a framework table, use `extend!{}` in an entity file:

```rust
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        avatar: image [upload_to: "avatars/"],
        bio: textarea,
        website: url [required],
    }
}
```

`makemigrations` detects these blocks and generates the corresponding `ALTER TABLE ADD COLUMN` statements.
The extension snapshot is stored in `migration/src/snapshots/runique/eihwaz_users.rs`.

**Extensible tables:**

```text
eihwaz_users · eihwaz_groupes · eihwaz_droits
eihwaz_sessions · eihwaz_users_groupes · eihwaz_groupes_droits
```

> Extending an unknown table triggers a **compile-time** error.

---

## Using a Custom User Table

If you prefer to manage your own user table, declare it in `.env`:

```env
RUNIQUE_USER_TABLE=my_users_table
```

`makemigrations` will then position `AdminTableMigration` right after the migration for `my_users_table`.
The FK in `eihwaz_users_groupes` will automatically target `my_users_table`.

> Default (no `RUNIQUE_USER_TABLE`): `eihwaz_users` is used.

---

## Other Migration Commands

```bash
sea-orm-cli migrate down --migration-dir migration/src   # Revert the last migration
sea-orm-cli migrate status --migration-dir migration/src # Check migration status
```

---

## Runique Wrapper — Atomic Rollback (advanced)

```bash
runique migration down --migrations migration/src <file>
runique migration down --migrations migration/src --batch <timestamp>
runique migration status --migrations migration/src
```

> These commands use the Runique batch system with transactional rollback.
> Prefer `sea-orm-cli` for the normal workflow.

---

> `runique makemigrations` is the only tool to use for **generating** migration files.
> Do not use `sea-orm-cli migrate generate`: Runique maintains its own snapshots and chronological order.

---

## See also

| Section | Description |
| --- | --- |
| [Database](/docs/en/installation/database) | SQLite, PostgreSQL, MariaDB |
| [Runique CLI](/docs/en/installation/cli) | Available commands |
| [Models](/docs/en/model) | `model!{}` DSL |

## Back to summary

- [Installation](/docs/en/installation)
