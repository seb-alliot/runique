# Project status — makemigrations work

What remains for `makemigrations` to generate every schema operation. What comes next for the project is in the [roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md).

**Last update**: November 8, 2026

## 4.1: complete makemigrations (in progress)

| Status | Step |
|:---:|---|
| ✓ | Plan, validation, atomic commit or rollback, snapshots |
| ✓ | Plain column rename (`renamed_from`) |
| ✓ | Destructive operations refused without `--force`: `DROP COLUMN`, type change, nullable to `NOT NULL`, reduced length or decimals, `DROP FK`, new `CASCADE` FK |
| ☐ | Drop a table when its model disappears: refused without `--force`, `DROP TABLE` in `up`, recreation in `down`, snapshot removed, reverse FK order |
| ☐ | Destructive changes: state what they affect, relations included |
| ☐ | Real `ALTER` on a type change (`USING` on PostgreSQL and MySQL, faithful snapshot; table rebuild on SQLite as an option) |
| ☐ | Nullable to `NOT NULL` with a default: fill existing `NULL`s before the constraint |
| ☐ | Column both renamed and modified (today only the rename applies, and the snapshot records the change as done) |
| ☐ | Detect a default value change |
| ☐ | Constraint names read from the database instead of assumed |
| ☐ | Identifier length limit per engine (63 for PostgreSQL, 64 for MariaDB/MySQL) |
| ☐ | Automatic creation and removal of many-to-many relation tables (DSL, migrations, macro) |
| ☐ | Honour `cascade`, `restrict` and `set_null` on those tables |
| ☐ | Table rename (`renamed_from`), and a clear message for a renamed model |

## 4.2: database-related features

| Status | Step |
|:---:|---|
| ☐ | `get_or_create`: fetch a row from conditions, or create it with default values |
| ☐ | `update_or_create`: update a row from conditions, or create it |
| ☐ | `exists()` and `last()` |
| ☐ | Loading relations in one query, for many-to-many |
| ☐ | Many-to-many access methods on the model |
| ☐ | `search!`: AND groups inside OR, negation of a condition or a group |

### Admin: many-to-many

| Status | Step |
|:---:|---|
| ☐ | Value selection in the form |
| ☐ | Display and filter in the list |
| ☐ | Links saved in the row's transaction, through the rights gate, recorded in the history |
