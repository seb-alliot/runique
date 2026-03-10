# Migrations (SeaORM)

## Two-Step Workflow

### 1. Generate Migration Files

`runique makemigrations` reads your entities declared in `src/entities` and generates the corresponding migration files:

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

### 2. Apply Migrations

Via the SeaORM CLI (recommended):

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

---

## Other Migration Commands

```bash
sea-orm-cli migrate down --migration-dir migration/src   # Revert the last migration
sea-orm-cli migrate status --migration-dir migration/src # Check migration status
```

---

## Runique Wrapper (advanced)

The following commands exist in the Runique CLI but **bypass SeaORM's chronological tracking**:

```bash
runique migration up --migrations migration/src
runique migration down --migrations migration/src
runique migration status --migrations migration/src
```

> ⚠️ These commands do not update SeaORM's migration tracking table. Use only if you know what you are doing — prefer `sea-orm-cli` for the normal workflow.

---

> ⚠️ `runique makemigrations` is the only tool to use for **generating** migration files. Do not use `sea-orm-cli migrate generate`: the Runique system maintains a chronological order and snapshots that the SeaORM CLI is not aware of.

---

## See also

| Section | Description |
| --- | --- |
| [Database](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/database/database.md) | SQLite, PostgreSQL |
| [Runique CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/cli/cli.md) | Available commands |

## Back to summary

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)
