# Project status — 4.1 and 4.2

What remains for the next two versions. An item may move from one version to the other along the way, when they have to ship together. What comes next for the project is in the [roadmap](https://github.com/seb-alliot/runique/blob/main/ROADMAP.md).

**Last update**: November 21, 2026

## 4.1: database infrastructure (in progress)

### makemigrations

| Status | Step |
|:---:|---|
| ✓ | Plan, validation, atomic commit or rollback |
| ✓ | Plain column rename (`renamed_from`) |
| ✓ | Destructive operations refused without `--force`: `DROP COLUMN`, type change, nullable to `NOT NULL`, reduced length or decimals, `DROP FK`, new `CASCADE` FK |
| ✓ | Length limit of index and foreign key names: refused beyond 63 characters, on every engine |
| ✓ | Connection to the database before any computation: `makemigrations` stops with an error when it does not answer |
| ✓ | One connection rule for the app and the commands: `DATABASE_URL` first, fallback on the `DB_*` keys when it is incomplete or doesn't answer; keys without `DB_ENGINE` refused; user and password of the keys encoded |
| ✓ | A database that doesn't match the project is refused: a migration recorded in the database and missing from `lib.rs` stops the command |
| ✓ | Snapshots checked before any computation: a snapshot that can't be read, or named otherwise than its table, stops the command |
| ✓ | Generated migration marked "pending": run again before it is applied, `makemigrations` recomputes it instead of stacking another one |
| ✓ | Two parallel changes to the same table: refused, with the tables involved (each migration notes the one it starts from) |
| ✓ | Drop a table when its model disappears: refused without `--force`, `DROP TABLE IF EXISTS` in `up`, recreation in `down`, reverse FK order, enum types removed once no column uses them; under SQLite, removing the key of a table that stays waits for the table rebuild |
| ✓ | `managed: false` in a model's `meta`: `makemigrations` no longer generates or drops anything for its table, and recalls the paused tables at every run |
| ✓ | Table and list field rename (`renamed_from`), with indexes, keys and list tables; the directive is still to be removed from the model after `migration up` |
| ☐ | Destructive changes: state what they affect, relations included |
| ☐ | Real `ALTER` on a type change, after `--force` (`USING` on PostgreSQL and MySQL, faithful snapshot) |
| ☐ | SQLite: table rebuild when a column is modified, or a foreign key is added or removed (today the column change is skipped) |
| ☐ | Nullable to `NOT NULL` with a default: fill existing `NULL`s before the constraint |
| ☐ | Required field added to a table that already holds rows: a choice on the console, give the value of the existing rows or do nothing |
| ☐ | Column both renamed and modified (today only the rename applies, and the snapshot records the change as done) |
| ☐ | Detect a default value change |
| ☐ | Constraint names read from the database instead of assumed |
| ☐ | Enums: error when two models declare the same name with different values; removed value handled |
| ☐ | Removed `extend!{}` block: its columns are removed from the database, the framework's table stays |

### migration up and down

| Status | Step |
|:---:|---|
| ☐ | All-or-nothing `migration up`: when a migration fails, none of that run stays applied (transaction on PostgreSQL and SQLite, compensation on MySQL) |
| ✓ | Snapshots written when applied: `makemigrations` no longer writes any, `migration up` writes the one of each table it changed once the run succeeded and removes the "pending" mark |
| ☐ | Snapshots brought back by `migration down` |
| ☐ | Tables already in the database without being recorded: checked, then recorded without being recreated |
| ☐ | `migration down`: undoes the last migration, leaving nothing half done, and removes the generated migration; also removes a generated migration not applied yet |
| ☐ | `makemigrations` and `migration up` cannot run at the same time |

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
