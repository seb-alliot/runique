# Migrations (SeaORM)

## Two-Step Workflow

### 1. Generate Migration Files

`runique makemigrations` reads your entities declared in `src/entities` and generates the corresponding migration files:

```bash
runique makemigrations --entities src/entities --migrations migration/src
```

### 2. Apply Migrations

Via the SeaORM CLI:

```bash
sea-orm-cli migrate up --migration-dir migration/src
```

Or via the Runique wrapper:

```bash
runique migration up --migrations migration/src
```

---

## Other Migration Commands

```bash
runique migration down --migrations migration/src    # Revert the last migration
runique migration status --migrations migration/src  # Check migration status
```

---

## See also

| Section | Description |
| --- | --- |
| [Database](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/database/database.md) | SQLite, PostgreSQL |
| [Runique CLI](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/cli/cli.md) | Available commands |

## Back to summary

- [Installation](https://github.com/seb-alliot/runique/blob/main/docs/en/installation/01-installation.md)
