# `model!` DSL & `extend!`

## Exposed macros

- `model! { ... }` — declares a model (SeaORM entity + migrations + admin form)
- `extend! { ... }` — adds columns to an existing framework table
- `#[form(...)]` — links a Rust form to a `model!` (see [Forms & concepts](/docs/en/model/forms))

All available via `use runique::prelude::*`.

---

## `model!` structure

The parser expects blocks **in this strict order** (optional blocks may be absent but not reordered):

```rust
model! {
    ModelName,              // 1. Name (PascalCase)
    table: "table_name",   // 2. SQL table name
    pk: field => type,     // 3. Primary key
    enums: { ... },        // 4. Optional — local enums
    { ... },               // 5. Fields — anonymous block, semantic types
    relations: { ... },    // 6. Optional — SeaORM relations
    meta: { ... },         // 7. Optional — constraints & ordering
}
```

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        title:      text     [required, max_length: 150],
        content:    textarea [required],
        is_active:  bool     [default: true],
        created_at: datetime [auto_now],
    }
}
```

> The field block is **always** an anonymous `{ ... }` block, never prefixed with a `fields:`
> keyword. Don't confuse this with `extend!{}`, which *does* require `fields:` — two different
> macros, two different grammars (see below).

---

## Primary key (`pk`)

```
pk: field_name => type
```

| Type   | Postgres SQL            | MySQL SQL                | Auto-increment | Creation                       |
|--------|--------------------------|----------------------------|----------------|----------------------------------|
| `i32`  | `SERIAL`                | `INT AUTO_INCREMENT`       | ✅ Yes          | DB sequence                      |
| `i64`  | `BIGSERIAL`             | `BIGINT AUTO_INCREMENT`    | ✅ Yes          | DB sequence                      |
| `uuid` | `UUID`                  | `VARCHAR(36)`               | ❌ No           | `Uuid::now_v7()` in Rust          |
| `Pk`   | alias `i32`/`i64`/`Uuid`| same                        | depends on type | depends on the active feature   |

**The `Pk` alias** defers to the application's global type, resolved by a Cargo feature —
**only one** may be active at a time (`compile_error!` if two are declared together):

```toml
[dependencies]
# nothing declared → Pk = i32 (default)
runique = { version = "3.0.0", features = ["postgres", "big-pk"] }    # Pk = i64
runique = { version = "3.0.0", features = ["postgres", "pk-uuid"] }   # Pk = Uuid (generated via Uuid::now_v7())
```

Use `big-pk` when you expect more than ~2 billion rows in a table, or when you need to interoperate with an existing schema using `BIGINT` primary keys. Use `pk-uuid` for non-sequential identifiers (multi-tenant setups, client-side generation, exposing ids publicly without leaking row volume).

### `Pk` on a regular field (not just the primary key)

The `Pk` keyword can also be used on an **ordinary FK field**, not only in `pk: id => Pk`.
This is the recommended way to declare a column referencing another table's primary key: it
then automatically follows the same feature, never drifting out of sync if you switch
`big-pk`/`pk-uuid` later.

```rust
model! {
    Chapter,
    table: "chapters",
    pk: id => Pk,
    {
        course_id: Pk [required],   // automatically follows Course.id's type
        title:     text [required],
    },
    relations: {
        belongs_to: Course via course_id,
    }
}
```

**Avoid**: declaring an FK with a fixed type (`course_id: int [required]`) when the
referenced table uses `pk: id => Pk`. It compiles and works as long as `Pk` is `i32`, but
silently drifts the moment a feature (`big-pk`/`pk-uuid`) changes — the same bug class
documented below for `big-pk`. Using `Pk` on the FK field removes the risk at the source, with
no conversion code (`.try_into()`) to maintain.

**Constraint when enabling `big-pk`/`pk-uuid`**: every FK column pointing at a `Pk` primary key
must stay consistent with it. The `course_id: Pk` form (above) guarantees this automatically; a
column fixed as `bigint`/`int`/`uuid` must be updated manually if you switch feature afterward.

> **The `big-pk`/`pk-uuid` choice must be made before the first migration.**
> Once migrations have been applied, switching mode is a breaking change: the database columns
> already have a concrete type, and changing the feature only changes the Rust type — the
> schema stays untouched. Switching after the fact requires a manual migration to `ALTER` every
> PK and FK column, risking truncation (`big-pk` → default) or total format incompatibility
> (`pk-uuid` ↔ any integer type). Pick one mode at project start.

---

## Field types

The Rust type shown is the one of a NOT NULL column; a `nullable` column gives `Option<T>`.

| DSL type           | Generated Rust type       | SQL column created by `makemigrations` |
|--------------------|---------------------------|---------------------------------|
| `text`             | `String`                  | `VARCHAR`, or `VARCHAR(n)` with `max_length: n` |
| `char`             | `String`                  | `VARCHAR`                       |
| `email`            | `String`                  | `VARCHAR(254)` — format validated |
| `password`         | `String`                  | `VARCHAR` — hashed automatically |
| `richtext`         | `String`                  | `TEXT` — HTML editor (sanitized) |
| `textarea`         | `String`                  | `TEXT` — multi-line             |
| `url`              | `String`                  | `VARCHAR` — format validated    |
| `slug`             | `String`                  | `VARCHAR`                       |
| `color`            | `String`                  | `VARCHAR` — hex color           |
| `phone`            | `String`                  | `VARCHAR(20)`, or `VARCHAR(n)` with `max_length: n` |
| `i8`               | `i8`                      | `TINYINT`                       |
| `i16`              | `i16`                     | `SMALLINT`                      |
| `int`              | `i32`                     | `INTEGER`                       |
| `bigint`           | `i64`                     | `BIGINT`                        |
| `u32`              | `u32`                     | `INTEGER UNSIGNED`              |
| `u64`              | `u64`                     | `BIGINT UNSIGNED`               |
| `f32`              | `f32`                     | `FLOAT`                         |
| `float`            | `f64`                     | `DOUBLE`                        |
| `percent`          | `f64`                     | `DOUBLE` — form bounded to 0–100 |
| `decimal`          | `Decimal`                 | `DECIMAL`                       |
| `bool`             | `bool`                    | `BOOLEAN`                       |
| `date`             | `NaiveDate`                | `DATE`                          |
| `time`             | `NaiveTime`                | `TIME`                          |
| `datetime`         | `NaiveDateTime`            | `DATETIME`                      |
| `timestamp`        | `NaiveDateTime`            | `DATETIME`                      |
| `timestamp_tz`     | `DateTime<Utc>`            | `TIMESTAMP WITH TIME ZONE`      |
| `uuid`             | `Uuid`                     | `UUID`                          |
| `Pk`               | `i32`/`i64`/`Uuid`         | depends on the feature — see above |
| `json`             | `serde_json::Value`        | `JSON`                          |
| `json_binary`      | `serde_json::Value`        | `JSON`                          |
| `binary`           | `Vec<u8>`                  | `BINARY(n)` — 255 by default, `max_length: n` |
| `var_binary`       | `Vec<u8>`                  | `VARBINARY(n)` — 255 by default, `max_length: n` |
| `blob`             | `Vec<u8>`                  | `BLOB`                          |
| `ip`               | `String`                   | `VARCHAR` — format validated    |
| `cidr`             | `String`                   | `VARCHAR`                       |
| `mac_address`      | `String`                   | `VARCHAR`                       |
| `interval`         | `String`                   | `VARCHAR`                       |
| `image`            | `String`                   | `VARCHAR` — file path           |
| `document`         | `String`                   | `VARCHAR` — file path           |
| `file`             | `String`                   | `VARCHAR` — file path           |
| `choice`           | `EnumName`                 | the enum's column — requires `enum(EnumName)`, rendered as `<select>` |
| `radio`            | `EnumName`                 | same as `choice`, rendered as radio buttons |
| `multichoice`      | — (list, see below)        | a table of its own `{table}_{field}` — rendered as `<select multiple>` |
| `checkbox`         | — (list, see below)        | same as `multichoice`, rendered as checkboxes |

A `VARCHAR` without a length takes the engine's default: unbounded on Postgres, `VARCHAR(255)`
on MySQL/MariaDB, `TEXT` on SQLite. `ip`, `cidr`, `mac_address` and `interval` stay `VARCHAR`:
their native types only exist on Postgres.

**Engine limits**, refused at compile time when a single engine feature is enabled: `i8` and
`u32` on Postgres, `u64` on Postgres and SQLite (the value can't be read back into the Rust
type). On MySQL/MariaDB, `binary` is a **fixed-length** column padded with `0x00`: use
`var_binary` to read back exactly the bytes written.

> **Not available**: inline `decimal(precision, scale)` (e.g. `decimal(10, 2)`) has no current
> equivalent — only plain `decimal` is supported. Workaround: apply precision/scale in
> application-level validation rather than in the schema.

---

## Field options

Inside a `[...]` block, comma-separated, value after `:` when the option takes one:

```rust
username: text [required, max_length: 150, unique],
```

| Option                    | Description                                                        |
|---------------------------|----------------------------------------------------------------------|
| `required`                | Field **mandatory in the form**. Says nothing about the column     |
| `nullable`                | `NULL` column — Rust type `Option<T>`                              |
| `unique`                  | `UNIQUE` constraint                                                |
| `max_length: n`           | Max length: validation **and** column size (`VARCHAR(n)`, `BINARY(n)`, `VARBINARY(n)`) |
| `min_length: n`           | Min length (validation)                                            |
| `min: n` / `max: n`       | Integer bounds (validation)                                        |
| `min: n.0` / `max: n.0`   | Float bounds (validation)                                          |
| `default: value`          | SQL default value (`true`, `0`, `"draft"`, etc.)                   |
| `auto_now`                | Filled on creation — excluded from forms                           |
| `auto_now_update`         | Filled on every save — excluded from forms                         |
| `readonly`                | Excluded from the generated migration (column exists on the Rust side, not managed by `derive_form`) |
| `label: "str"`            | Custom label in forms                                              |
| `upload_to: "path"`       | File field — upload directory                                      |
| `max_size: n MB`          | File field — max size (`KB`/`MB`/`GB`)                             |
| `rows: n`                 | `textarea`/`richtext` — widget height                              |
| `step: n`                 | Numeric fields — widget step                                       |
| `enum(EnumName)`          | Binds the field to an enum declared in `enums:`                    |
| `renamed_from: "x"`       | Renames the column (see below)                                     |
| `skip`                    | Excluded from generated forms                                      |
| `no_hash`                 | `password` fields only — disables automatic hashing                |

An unknown attribute, or one not valid for the type, is a **compile error** naming the field and
the attribute. `makemigrations` reads the DSL with the same parser as the macro: a model one
refuses, the other refuses too, with file, line and column.

> **A column's name decides nothing.** A `created_at` column without `auto_now` has no default,
> a `cache_key` column is migrated like any other: only attributes count.

> **`readonly`** (DB-level) is distinct from `#[form]`'s own `readonly`
> (`field_readonly()`, disables a field in the HTML rendering of one specific form instance).
> `readonly` on the model field excludes the column from the generated migration;
> `field_readonly()` just disables a widget at runtime. Both can coexist.

### Nullability

A column is **NOT NULL** unless declared `nullable`. `required` only concerns the form. The
parser refuses, at compile time:

- `required` together with `nullable`;
- `nullable` on `auto_now` / `auto_now_update`: the framework always fills them;
- a field that isn't text-like (number, date, choice, uuid, json…) declared without `required`,
  `nullable` or `default` — an optional field left empty would have nothing to store in a NOT
  NULL column. Text, file, binary and `bool` fields can stay optional: they always submit a
  storable value (empty text, no file, unchecked box).

```rust
{
    title:        text  [required],          // NOT NULL, mandatory in the form
    subtitle:     text,                      // NOT NULL, optional (empty text)
    summary:      text  [nullable],          // may be NULL — Option<String>
    score:        int   [default: 0],        // NOT NULL, 0 when nothing is typed
    published_on: date  [nullable],          // may be NULL — Option<NaiveDate>
}
```

### `auto_now` / `auto_now_update`

Filled by the entity itself (`ActiveModelBehavior::before_save`), the same way on every engine:
`auto_now` on creation unless already set, `auto_now_update` on every save. The migration also
adds `DEFAULT CURRENT_TIMESTAMP`, for an insert made outside SeaORM. The Rust type is `T` (never
`Option<T>`), and these fields are excluded from `admin_from_form` and `admin_partial_update`.

### Column lengths

`max_length` sets the column size, tracked by the snapshots: growing it produces a plain
`ALTER`, shrinking it needs `--force` (longer values would be cut or refused). A snapshot written
before 3.0 takes the model's lengths once, without producing a migration.

### Bounds and `customize`

`min_length`, `max_length`, `min` and `max` reach the forms generated from the model. A form may
**tighten** them in `customize`, never **loosen** them: a `customize` that lengthens a
`max_length`, lowers a `min`, etc. panics when the form is built, with a message naming the field
and the bound.

### Renaming a column — `renamed_from`

Renaming a field without this option produces a `DROP COLUMN` + `ADD COLUMN` → **data loss**.
The tool is non-interactive and cannot guess intent: you must state it explicitly.

```rust
// before:  job_title: text,
// after:
title: text [renamed_from: "job_title"],
```

`makemigrations` then emits `ALTER TABLE … RENAME COLUMN job_title TO title` (supported by
PostgreSQL, MySQL/MariaDB and SQLite), with no data loss. The attribute is a migration-only
directive: it has no effect on the generated entity or form. Guard: if the old column still
exists in the snapshot (stale hint), no rename is emitted.

Works in both `model!{}` and `extend!{}`.

---

## Enums

Declared in a separate `enums: { ... }` block, then referenced via `enum(EnumName)`.

```rust
model! {
    Order,
    table: "orders",
    pk: id => i32,
    enums: {
        OrderStatus: [
            Pending    = ("pending",    "Pending"),
            InProgress = ("in_progress","In progress"),
            Delivered  = ("delivered",  "Delivered"),
            Cancelled  = ("cancelled",  "Cancelled"),
        ],
        Priority: i32 [Low = 0, Normal = 1, High = 2, Urgent = 9],
    },
    {
        status:   choice [enum(OrderStatus), required],
        priority: choice [enum(Priority), required],
    },
}
```

### Four variant forms — never confuse these

> **Watch out, common trap**: `:` and `=` do **not** do the same thing. A single symbol
> difference completely changes the behavior, with no compile error to warn you. Always check
> against this table, never guess by analogy.

| Syntax                                | Stored DB value        | Displayed label (`Display`)                        |
|-----------------------------------------|--------------------------|--------------------------------------------------------|
| `Variant`                               | `"Variant"` (the name)   | `"Variant"` — falls back to the DB value                 |
| `Variant: "Label"`                      | `"Variant"` (**unchanged**) | `"Label"`                                            |
| `Variant = "db_value"`                  | `"db_value"`              | `"db_value"` — falls back to the DB value, **not** `Variant` |
| `Variant = ("db_value", "Label")`       | `"db_value"`              | `"Label"`                                                |

Rule summary:
- `:` (colon) affects **only the display** — the stored value is always the variant name.
- `=` alone (no parentheses) affects **only the stored value** — display falls back to it, never to the variant name.
- `= (a, b)` sets both independently — the only form that lets a variant name, a DB value and a label all differ.

**The label is purely cosmetic.** It affects neither:
- the actual storage (`#[sea_orm(string_value = ...)]` / `#[sea_orm(num_value = ...)]` always use the DB value, never the label),
- nor parsing comparisons (`FromStr` compares against the DB value and the Rust variant name first — the label is only accepted as a fallback, and only when it differs from the other two).

Changing a label (`Published: "Published"` → `Published: "Live"`) therefore has **no** impact on stored data or on code comparing enum values.

> **The DB value is stored exactly as written.** No automatic transformation.

### Backing types

| Syntax                | DB storage                                        |
|-------------------------|------------------------------------------------------|
| `EnumName: [A, B]`      | Native `ENUM` (Postgres) or `VARCHAR` (MySQL/SQLite)  |
| `EnumName: i8 [...]`    | `TINYINT` — refused on Postgres                       |
| `EnumName: i16 [...]`   | `SMALLINT`                                            |
| `EnumName: i32 [...]`   | `INTEGER`                                             |
| `EnumName: i64 [...]`   | `BIGINT`                                              |

For an integer enum, `=` sets the stored number, and **every variant needs one** (`Low = 0` or
`Low = (0, "Low")`), within the type's range, with no duplicate. Otherwise it's a compile error:
two variants sharing a value would be read back as one another.

### Generated methods

| Method | Return | Description |
|--------|--------|-------------|
| `.to_string()` | `String` | Display label |
| `.db_value()` | `&'static str` / `i8` … `i64` | Exact DB value |
| `.form_value()` | `&'static str` | Value a form sends (stored value for a text enum, variant name for an integer enum) |
| `::from_str(s)` / `.parse()` | `Result<Self, ()>` | Parse from DB value, label, or variant name |
| `::iter()` | `impl Iterator<Item = Self>` | Iterate over all variants |

```rust
use sea_orm::Iterable;

let s = OrderStatus::Pending;
s.db_value()   // → "pending"
s.to_string()  // → "Pending"

// For a <select>
let options: Vec<(String, String)> = OrderStatus::iter()
    .map(|v| (v.db_value().to_string(), v.to_string()))
    .collect();

// Parse from a DB value
let status: Option<OrderStatus> = "pending".parse().ok();
```

**In Tera templates**, the comparison value must match **exactly** what is stored in the database (case-sensitive).

---

## File fields

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    {
        image:      image    [upload_to: "media/articles"],
        attachment: document [upload_to: "docs/"],
        upload:     file     [upload_to: "media/uploads"],
    },
}
```

| Type      | Allowed extensions             |
|-----------|--------------------------------|
| `image`   | `jpg jpeg png gif webp avif`   |
| `document`| `pdf doc docx txt odt`         |
| `file`    | no filter                      |

`upload_to:` is required for all three types. The path is relative to `MEDIA_ROOT`.

---

## Relations

```rust
relations: {
    belongs_to: course via course_id [cascade],     // foreign key
    has_many: Comment,
    has_many: Comment as comments,                  // optional alias
    has_one: Profile as profile,
    many_to_many: Role through UserRole via user_id,
}
```

| Type           | DB constraint   | Description                   |
|----------------|-----------------|--------------------------------|
| `belongs_to`   | ✅ `FOREIGN KEY` + index | N-1 relation         |
| `has_many`     | ❌ code only     | 1-N relation                  |
| `has_one`      | ❌ code only     | 1-1 relation                  |
| `many_to_many` | ❌ code only     | N-N through a pivot table declared separately |

### `belongs_to` — the foreign key

`belongs_to: target via column [on_delete, on_update]` is the **only** way to declare a foreign
key. It produces both the SQL constraint and the SeaORM relation:

- **`target`** is the module of the target entity — the file `target.rs` in `src/entities/` — or
  a framework table (`eihwaz_users`, …). The key references that model's **real table and real
  primary key**, whatever its name (`pk: code => i32` included). An unresolved target is a
  `makemigrations` error.
- **`column`** must be a declared field of the model (preferably `Pk`, see above).
- **Actions** in brackets: the first for `ON DELETE`, the second (optional) for `ON UPDATE` —
  `cascade`, `set_null`, `restrict`, `set_default`, `no_action` (default). An unknown action is a
  compile error, and `set_null` requires a `nullable` column.
- **Index**: `makemigrations` creates an index `idx_<table>_<column>` on every `belongs_to`
  column, as Django does — unless the column is already `unique` or leads an index declared in
  `meta`.
- **Cycles**: two new tables referencing each other are handled. On SQLite, the key stays in the
  `CREATE TABLE`; on Postgres and MySQL, the one closing the cycle is added by an `ALTER TABLE`
  once the other table exists.

> Since 3.0, the former `fk(table.col, action)` field option no longer exists: it is a compile
> error pointing to `belongs_to`.

---

## List fields — `multichoice` and `checkbox`

A list field holds **several values of an enum**. `multichoice` is rendered as a
`<select multiple>`, `checkbox` as checkboxes; both are stored and used the same way.

```rust
model! {
    Book,
    table: "books",
    pk: id => Pk,
    enums: { Genre: [Novel, Crime, Youth = ("youth", "Youth")] },
    {
        title:  text [required],
        genres: checkbox [enum(Genre), required],   // required = at least one value
    }
}
```

- **Storage**: no column in `books`, and no `genres` field in the `Model`. The values live in a
  `books_genres` table created by `makemigrations`: `id`, `owner_id` (FK `ON DELETE CASCADE` to
  the owning row), `value` (the enum's column), a unique index `(owner_id, value)` and an index
  `(value, owner_id)` for filters.
- **Attributes**: the enum is mandatory; only `required`, `enum(...)` and `label` are accepted
  (an empty list already says "none"). A list field can't be referenced in `meta` or by
  `belongs_to`, and isn't accepted in `extend!{}`. A name that would hide a SeaORM method (`get`,
  `set`, `delete`, `find_related`…) is refused.

### Reading and writing

```rust
let genres: Vec<Genre> = book.genres(&db).await?;          // in the order they were set
book.set_genres(&db, [Genre::Novel, Genre::Crime]).await?; // replaces, in a transaction, no duplicates
let by_book = book::Model::load_genres(&db, &books).await?; // a whole page: 1 query
```

### Filtering

```rust
search!(book::Entity => Genres has Genre::Novel)
search!(book::Entity => Genres has_any [Genre::Novel, Genre::Crime])
search!(book::Entity => Genres has_all [Genre::Novel, Genre::Crime])
search!(book::Entity => !Genres has Genre::Novel, Title icontains "night")

book::Entity::objects.filter(book::List::Genres.has(Genre::Novel))   // without search!
```

The filters are portable subqueries (Postgres, MySQL/MariaDB, SQLite) and are typed: passing a
value of another enum doesn't compile.

### Postgres: rows and lists in one query

With the `postgres` feature, `fetch_with` returns the rows of a query **with** their list, in a
single round trip (the query's filter and order are kept):

```rust
let page: Vec<(book::Model, Vec<Genre>)> =
    book::List::Genres.fetch_with(&db, book::Entity::find().limit(50)).await?;
```

On another engine it returns an error; use `load_genres` after reading the page.

### Forms and admin

The admin shows the field (checkboxes or multiple select), saves the row and its list in the
same transaction, and pre-fills the edit form. In a form, the checked values are read with
`cleaned_enums::<Genre>("genres")`.

---

## Meta

```rust
meta: {
    ordering: [-created_at, title],
    unique_together: [(slug, lang)],
    indexes: [(lang, sort_order)],
    verbose_name: "Article",
    verbose_name_plural: "Articles",
}
```

| Key                   | Syntax                | Effect                                      |
|-----------------------|-----------------------|-----------------------------------------------|
| `ordering`            | `[field, -field]`     | Default sort order, `-` = `DESC`              |
| `unique_together`     | `[(col1, col2)]`      | Multi-column `UNIQUE` constraint              |
| `indexes`             | `[(col1, col2)]`      | Multi-column simple index                     |
| `verbose_name`        | `"string"`            | Singular name in the admin interface          |
| `verbose_name_plural` | `"string"`            | Plural name in the admin interface            |

---

## `extend!{}` — extending framework tables

Adds columns to a Runique table and generates a complete SeaORM entity for that table.

`extend!{}` produces two things:

1. **SQL schema** — `makemigrations` detects the block and generates `ALTER TABLE ADD COLUMN` statements
2. **Full entity** — `Model`, `Column`, `Entity`, `AdminForm`, `admin_from_form`, `admin_partial_update` covering **all** columns of the table (base columns + extended columns)

```rust
// src/entities/user_profile.rs
use runique::prelude::*;

extend! {
    table: "eihwaz_users",
    fields: {
        bio:         textarea [nullable],
        avatar:      image    [nullable, upload_to: "avatars/"],
        website:     url      [nullable],
        phone:       phone    [nullable],
        birth_date:  date     [nullable],
        is_verified: bool     [default: false],
    }
}
```

> `extend!{}` **always** requires the `fields:` keyword before the field block — unlike
> `model!{}` (direct anonymous block). These are two different macros with two different
> grammars; don't carry syntax from one over to the other.

Allowed tables: `eihwaz_users`, `eihwaz_groupes`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Any other name causes a compile-time error.

Fields in `extend!{}` use the same types, options and nullability rules as `model!` (including `renamed_from`). Columns added to an existing table get a value for the rows already there: declare them `nullable` or with a `default`. No `relations:` block and no list field (`multichoice`/`checkbox`) inside `extend!{}`.

### Enums in `extend!{}`

`extend!{}` accepts an optional `enums: { ... }` block (between `table:` and `fields:`), identical to `model!`. A `choice [enum(EnumName)]` column generates the Rust enum type, the typed column and the populated `ChoiceField`:

```rust
extend! {
    table: "eihwaz_users",
    enums: {
        Seniority: [Junior="junior", Mid="mid", Senior="senior", Lead="lead"],
    },
    fields: {
        job_title: text [nullable],
        seniority: choice [enum(Seniority), nullable],
    }
}
```

`makemigrations` emits the column (on PostgreSQL, a `CREATE TYPE … AS ENUM`; elsewhere a native `VARCHAR`/`ENUM`).

### Full workflow

```bash
# 1. Declare the extension in src/entities/
# 2. Generate the migration
runique makemigrations

# 3. Apply
runique migration up

# 4. Register in admin!{} (src/admin.rs)
```

```rust
admin! {
    configure {
        users: { hidden: true }   // hides the builtin "Users" panel
    }
    user_profile: user_profile::Model => user_profile::AdminForm {
        title: "User profiles",
        list_display: [
            ["username", "User"],
            ["bio", "Bio"],
            ["is_verified", "Verified"],
        ],
    }
}
```

### What is generated

| Symbol | Description |
| ------ | ----------- |
| `Model` | Struct with all columns (base + extended) |
| `Column` | SeaORM column enum |
| `Entity` | Full `EntityTrait` — usable with `search!` |
| `AdminForm` | Admin form covering all columns |
| `admin_from_form` | Builds an `ActiveModel` from form data |
| `admin_partial_update` | Builds a partial `ActiveModel` for updates |

### Queries from views

The generated entity is a standard SeaORM `EntityTrait` — `search!` works directly:

```rust
// All verified profiles
let profiles = search!(user_profile::Entity => IsVerified eq true).all(&db).await?;

// Multi-column search
let results = search!(user_profile::Entity => or(Username icontains q, Bio icontains q)).all(&db).await?;
```

### Relations targeting a framework table

An entity points to a framework table by its table name, in the usual `relations:` block of `model!{}`:

```rust
model! {
    Article,
    table: "articles",
    pk: id => Pk,
    { author_id: Pk [required] },
    relations: {
        belongs_to: eihwaz_users via author_id [cascade],
    }
}
```

---

## See also

| Section | Description |
| --- | --- |
| [Generation & ModelSchema](/docs/en/model/generation) | Generated code, `schema()`, `ModelSchema` |
| [Forms & concepts](/docs/en/model/forms) | `#[form(...)]`, model/form binding |

## Back to summary

- [Models](/docs/en/model)
