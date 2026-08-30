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
runique = { version = "2.2.0", features = ["big-pk"] }    # Pk = i64
runique = { version = "2.2.0", features = ["pk-uuid"] }   # Pk = Uuid (generated via Uuid::now_v7())
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

| DSL type           | Generated Rust type       | SQL column                       |
|----------------------|---------------------------|------------------------------------|
| `text`               | `String`                  | `VARCHAR(255)` or `VARCHAR(n)` if `max_length: n` |
| `char`               | `String`                  | `CHAR`                             |
| `email`              | `String`                  | `VARCHAR(254)` — validated format  |
| `password`           | `String`                  | `VARCHAR(255)` — automatically hashed |
| `richtext`           | `String`                  | `TEXT` — HTML editor                |
| `textarea`           | `String`                  | `TEXT` — multi-line                 |
| `url`                | `String`                  | `VARCHAR(255)` — validated format   |
| `slug`               | `String`                  | `VARCHAR(255)`                      |
| `color`              | `String`                  | `VARCHAR(255)` — hex color          |
| `phone`              | `String`                  | `VARCHAR(20)` or `VARCHAR(n)` if `max_length: n` |
| `i8`                 | `i8`                      | `TINYINT`                           |
| `i16`                | `i16`                     | `SMALLINT`                          |
| `int`                | `i32`                     | `INTEGER`                           |
| `bigint`             | `i64`                     | `BIGINT`                            |
| `u32`                | `u32`                     | `INTEGER UNSIGNED`                  |
| `u64`                | `u64`                     | `BIGINT UNSIGNED`                   |
| `f32`                | `f32`                     | `FLOAT`                             |
| `float`              | `f64`                     | `DOUBLE`                            |
| `percent`            | `f64`                     | `DOUBLE` — stored as float          |
| `decimal`            | `Decimal`                 | `DECIMAL`                           |
| `bool`               | `bool`                    | `BOOLEAN`                           |
| `date`               | `NaiveDate`                | `DATE`                              |
| `time`               | `NaiveTime`                | `TIME`                              |
| `datetime`           | `NaiveDateTime`            | `DATETIME`                          |
| `timestamp`          | `NaiveDateTime`            | `TIMESTAMP`                         |
| `timestamp_tz`       | `DateTime<Utc>`            | `TIMESTAMPTZ`                       |
| `uuid`               | `Uuid`                     | `UUID`                              |
| `Pk`                 | `i32`/`i64`/`Uuid`         | depends on the active feature — see above |
| `json`               | `serde_json::Value`        | `JSON`                              |
| `json_binary`        | `serde_json::Value`        | `JSON BINARY`                       |
| `binary`             | `Vec<u8>`                  | `BINARY` — size via `max_length: n` |
| `var_binary`         | `Vec<u8>`                  | `VARBINARY(n)` — `max_length: n` required |
| `blob`               | `Vec<u8>`                  | `BLOB`                              |
| `ip`                 | `String`                   | `INET`                              |
| `cidr`               | `String`                   | `CIDR`                              |
| `mac_address`        | `String`                   | `MACADDR`                           |
| `interval`           | `String`                   | `INTERVAL`                          |
| `image`              | `String`                   | `VARCHAR(255)` — file path          |
| `document`           | `String`                   | `VARCHAR(255)` — file path          |
| `file`               | `String`                   | `VARCHAR(255)` — file path          |
| `choice`             | `EnumName`                 | `VARCHAR` / native `ENUM` — requires `enum(EnumName)` |
| `radio`              | `EnumName`                 | Same as `choice`, different widget  |
| `checkbox`           | `EnumName`                 | Same as `choice`, different widget  |

`binary`/`var_binary` reuse the `max_length: n` option (same mechanism as `text` +
`max_length` → `VARCHAR(n)`) — there is no separate inline `binary(n)` syntax.

> **Not available**: inline `decimal(precision, scale)` (e.g. `decimal(10, 2)`) has no current
> equivalent — only plain `decimal` with no parameters is supported. Workaround: enforce
> precision/scale at the application-validation layer instead of in the schema.

---

## Field options

In a `[...]` block, comma-separated, value after `:` when the option takes one:

```rust
username: text [required, max_length: 150, unique],
```

| Option                    | Description                                                       |
|---------------------------|---------------------------------------------------------------------|
| `required`                | `NOT NULL` column + form validation                               |
| `nullable`                | `NULL` column — Rust type `Option<T>`                              |
| `unique`                  | `UNIQUE` constraint                                                |
| `max_length: n`           | Max length (validation + column size)                             |
| `min_length: n`           | Min length (validation)                                            |
| `min: n`                  | Min integer value (validation)                                     |
| `max: n`                  | Max integer value (validation)                                     |
| `min: n.0`                | Min float value (validation)                                       |
| `max: n.0`                | Max float value (validation)                                       |
| `default: value`          | SQL default value (`true`, `0`, `"draft"`, etc.)                  |
| `auto_now`                | Set to `NOW()` on every `INSERT` — excluded from forms             |
| `auto_now_update`         | Set to `NOW()` on every `UPDATE` — excluded from forms             |
| `readonly`                | Excluded from the generated migration (column exists in Rust, not managed by `derive_form`) |
| `label: "str"`            | Custom label in admin forms                                       |
| `help: "str"`             | Reserved — not yet wired to rendering                             |
| `upload_to: "path"`       | File field — upload directory                                     |
| `max_size: n MB`          | File field — max size (`KB`/`MB`/`GB`)                            |
| `rows: n`                 | `textarea`/`richtext` — widget height                              |
| `step: n`                 | Numeric fields — widget step                                       |
| `fk(table.col, action)`   | Foreign key constraint (see Relations)                             |
| `enum(EnumName)`          | Links the field to an enum declared in `enums:`                    |
| `renamed_from: "x"`       | Renames the column (see below)                                     |
| `skip`                    | Excluded from generated forms                                      |
| `no_hash`                 | `password` fields only — disables automatic hashing                |

> **`readonly`** (new, DB-level) is distinct from `#[form]`'s own `readonly`
> (`field_readonly()`, disables a field in the HTML rendering of one specific form instance).
> `readonly` on the model field excludes the column from the generated migration;
> `field_readonly()` just disables a widget at runtime. Both can coexist.

> **`auto_now` / `auto_now_update`**: excluded from `admin_from_form` and `admin_partial_update`. Their value is managed by the database only. They appear in `Model` and `Column` as `Option<T>`.

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
| `EnumName: i32 [...]`   | `INTEGER` — `=` then sets the numeric value, not a string |
| `EnumName: i64 [...]`   | `BIGINT` — same                                        |

### Generated methods

| Method | Return | Description |
|--------|--------|-------------|
| `.to_string()` | `String` | Display label |
| `.db_value()` | `&'static str` / `i32` / `i64` | Exact DB value |
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
    belongs_to: Model via fk_field,
    has_many: Model,
    has_many: Comments as user_comments,   // optional alias
    has_one: Profile as user_profile,
    many_to_many: Roles through UserRoles via self_id,
}
```

| Type           | DB constraint   | Description                   |
|----------------|-----------------|--------------------------------|
| `belongs_to`   | ❌ code only     | N-1 relation (SeaORM)         |
| `has_many`     | ❌ code only     | 1-N relation                  |
| `has_one`      | ❌ code only     | 1-1 relation                  |
| `many_to_many` | ❌ code only     | N-N via pivot table            |

> **Actual FK constraint**: the SQL `FOREIGN KEY` and its action (`cascade`, `restrict`, `set_null`, `set_default`) are declared on the `fk(table.col, action)` field option, not in the `relations:` block. The `relations:` block only generates SeaORM traits for object navigation.

Available FK actions on `fk(...)`: `cascade` · `restrict` · `set_null` · `set_default`

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
| `abstract`            | `true`                | Abstract model — no table generated           |

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
        bio:         textarea,
        avatar:      image  [upload_to: "avatars/"],
        website:     url,
        phone:       phone,
        birth_date:  date,
        is_verified: bool   [default: false],
    }
}
```

> `extend!{}` **always** requires the `fields:` keyword before the field block — unlike
> `model!{}` (direct anonymous block). These are two different macros with two different
> grammars; don't carry syntax from one over to the other.

Allowed tables: `eihwaz_users`, `eihwaz_groupes`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Any other name causes a compile-time error.

Fields in `extend!{}` use the same types and options as `model!` (including `renamed_from`). No `relations:` block inside `extend!{}` — relations are declared in the target `model!{}` with `has_many(user_profile)` etc.

### Enums in `extend!{}`

`extend!{}` accepts an optional `enums: { ... }` block (between `table:` and `fields:`), identical to `model!`. A `choice [enum(EnumName)]` column generates the Rust enum type, the typed column and the populated `ChoiceField`:

```rust
extend! {
    table: "eihwaz_users",
    enums: {
        Seniority: [Junior="junior", Mid="mid", Senior="senior", Lead="lead"],
    },
    fields: {
        job_title: text,
        seniority: choice [enum(Seniority)],
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
runique migrate

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
let profiles = search!(user_profile::Entity => is_verified eq true).fetch(&db).await?;

// Multi-column search
let results = search!(user_profile::Entity => or(username icontains q, bio icontains q)).fetch(&db).await?;
```

### Relations targeting the extended entity

Other entities can point to the extended entity via the usual `relations:` block in `model!{}`:

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    { author_id: int [required] },
    relations: {
        belongs_to: user_profile::Model via author_id,
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
