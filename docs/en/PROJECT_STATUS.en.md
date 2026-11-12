# Project status — 4.1 and 4.2

What remains for the next two versions. An item may move from one version to the other along the way, when they have to ship together. What comes next for the project is in the [roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md).

**Last update**: November 12, 2026

## 4.1: database infrastructure (in progress)

### makemigrations

| Status | Step |
|:---:|---|
| ✓ | Plan, validation, atomic commit or rollback, snapshots |
| ✓ | Plain column rename (`renamed_from`) |
| ✓ | Destructive operations refused without `--force`: `DROP COLUMN`, type change, nullable to `NOT NULL`, reduced length or decimals, `DROP FK`, new `CASCADE` FK |
| ✓ | Length limit of index and foreign key names: refused beyond 63 characters, on every engine |
| ☐ | Drop a table when its model disappears: refused without `--force`, `DROP TABLE` in `up`, recreation in `down`, snapshot removed, reverse FK order |
| ☐ | Destructive changes: state what they affect, relations included |
| ☐ | Real `ALTER` on a type change (`USING` on PostgreSQL and MySQL, faithful snapshot; table rebuild on SQLite as an option) |
| ☐ | Nullable to `NOT NULL` with a default: fill existing `NULL`s before the constraint |
| ☐ | Column both renamed and modified (today only the rename applies, and the snapshot records the change as done) |
| ☐ | Detect a default value change |
| ☐ | Constraint names read from the database instead of assumed |
| ☐ | Table rename (`renamed_from`), and a clear message for a renamed model |

### Queries

| Status | Step |
|:---:|---|
| ☐ | `get_or_create`: fetch a row from conditions, or create it with default values |
| ☐ | `update_or_create`: update a row from conditions, or create it |
| ☐ | `exists()` and `last()` |

### Limiters

| Status | Step |
|:---:|---|
| ☐ | Cap on the number of keys held in memory by `RateLimiter` and `LoginGuard` |
| ☐ | Tiered lockout: a duration that grows at each repeat (5 min, 15 min, 1 h by default), each tier configurable (`lockout_secs` deprecated, removed in 5.0) |

## 4.2: relations

| Status | Step |
|:---:|---|
| ☐ | Automatic creation and removal of many-to-many relation tables (DSL, migrations, macro) |
| ☐ | Honour `cascade`, `restrict` and `set_null` on those tables |
| ☐ | Loading relations in one query, for many-to-many |
| ☐ | Many-to-many access methods on the model |
| ☐ | `search!`: AND groups inside OR, negation of a condition or a group |

### Admin: many-to-many

| Status | Step |
|:---:|---|
| ☐ | Value selection in the form |
| ☐ | Display and filter in the list |
| ☐ | Links saved in the row's transaction, through the rights gate, recorded in the history |
