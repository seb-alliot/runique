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
    fields: { ... },       // 5. Fields (v1 or v2 syntax)
    relations: { ... },    // 6. Optional — SeaORM relations
    meta: { ... },         // 7. Optional — constraints & ordering
}
```

### Two field syntaxes

**Syntax v1 — explicit SQL types** (named `fields:` block):

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    fields: {
        title:      String   [required, max_len(150)],
        content:    text     [required],
        is_active:  bool,
        created_at: datetime [auto_now],
    },
}
```

**Syntax v2 — semantic types** (anonymous `{ ... }` block):

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

> In syntax v2, the `form_fields:` block is ignored — semantic types already carry widget information.

---

## Primary key (`pk`)

```
pk: field_name => type
```

| Type   | Postgres SQL        | MySQL SQL              | Auto-increment | Creation |
|--------|---------------------|------------------------|----------------|----------|
| `i32`  | `SERIAL`            | `INT AUTO_INCREMENT`   | ✅ Yes          | DB sequence |
| `i64`  | `BIGSERIAL`         | `BIGINT AUTO_INCREMENT`| ✅ Yes          | DB sequence |
| `uuid` | `UUID`              | `VARCHAR(36)`          | ❌ No           | `Uuid::new_v4()` in Rust |
| `Pk`   | alias `i32` or `i64`| same                   | ✅ Yes          | depends on `big-pk` feature |

**The `Pk` alias** resolves to `i32` by default, or `i64` if the `big-pk` feature is enabled:

```toml
[dependencies]
runique = { version = "2.1.15", features = ["big-pk"] }
```

Use `big-pk` when you expect more than ~2 billion rows in a table, or when you need to interoperate with an existing schema that uses `BIGINT` primary keys.

**Constraints when enabling `big-pk`:**

- Every FK column pointing to a `Pk` primary key must also be declared `bigint`, otherwise you get a type mismatch at compile time:

```rust
model! {
    Order,
    table: "orders",
    pk: id => Pk,
    fields: {
        user_id: bigint [required]   // must match users.id which is Pk (i64)
    },
}
```

- The admin daemon generates `parse::<Pk>()` by default in `admin.rs`, so the generated code automatically follows the feature — no manual adjustment needed.

- Seed files and any handwritten code that assigns `entity.id` (a `Pk`) to an `i32` FK field must use `.try_into().unwrap()` or change the FK column to `bigint`.

> **`big-pk` must be decided before the first migration.**
> Once migrations have been applied, switching between `big-pk` and the default (`i32`) is a breaking change: the database columns are already `INT` or `BIGINT`, and changing the feature flag alone only changes the Rust type — the schema stays untouched. Switching after the fact requires a manual migration to `ALTER` every PK and FK column, and risks data truncation if existing IDs exceed `i32::MAX`. Pick one mode at project start and keep it.

---

## Field types — syntax v1

Directly declared SQL types:

| DSL type          | Generated Rust type       | SQL column                |
|-------------------|---------------------------|---------------------------|
| `String`          | `String`                  | `VARCHAR(255)`            |
| `text`            | `String`                  | `TEXT`                    |
| `char`            | `String`                  | `CHAR`                    |
| `varchar(n)`      | `String`                  | `VARCHAR(n)`              |
| `i8`              | `i32`                     | `TINYINT`                 |
| `i16`             | `i32`                     | `SMALLINT`                |
| `i32`             | `i32`                     | `INTEGER`                 |
| `i64`             | `i64`                     | `BIGINT`                  |
| `u32`             | `u32`                     | `INTEGER UNSIGNED`        |
| `u64`             | `u64`                     | `BIGINT UNSIGNED`         |
| `f32`             | `f32`                     | `FLOAT`                   |
| `f64`             | `f64`                     | `DOUBLE`                  |
| `decimal`         | `Decimal`                 | `DECIMAL`                 |
| `decimal(p, s)`   | `Decimal`                 | `DECIMAL(p, s)`           |
| `bool`            | `bool`                    | `BOOLEAN`                 |
| `date`            | `NaiveDate`               | `DATE`                    |
| `time`            | `NaiveTime`               | `TIME`                    |
| `datetime`        | `NaiveDateTime`           | `DATETIME`                |
| `timestamp`       | `NaiveDateTime`           | `TIMESTAMP`               |
| `timestamp_tz`    | `NaiveDateTime`           | `TIMESTAMPTZ`             |
| `uuid`            | `Uuid`                    | `UUID`                    |
| `json`            | `serde_json::Value`       | `JSON`                    |
| `json_binary`     | `serde_json::Value`       | `JSON BINARY`             |
| `binary`          | `Vec<u8>`                 | `BINARY`                  |
| `binary(n)`       | `Vec<u8>`                 | `BINARY(n)`               |
| `var_binary(n)`   | `Vec<u8>`                 | `VARBINARY(n)`            |
| `blob`            | `Vec<u8>`                 | `BLOB`                    |
| `inet`            | `String`                  | `INET`                    |
| `cidr`            | `String`                  | `CIDR`                    |
| `mac_address`     | `String`                  | `MACADDR`                 |
| `interval`        | `String`                  | `INTERVAL`                |
| `enum(EnumName)`  | `EnumName`                | `INTEGER` / `ENUM` / `VARCHAR` |

---

## Field types — syntax v2 (semantic)

Automatically converted to SQL types:

| Semantic type   | Generated SQL                          | Notes                               |
|-----------------|----------------------------------------|-------------------------------------|
| `text`          | `VARCHAR(255)` or `VARCHAR(n)` if `max_length: n` |                        |
| `email`         | `VARCHAR(254)`                         | Validated email format              |
| `password`      | `VARCHAR(255)`                         | Automatically hashed                |
| `richtext`      | `TEXT`                                 | HTML editor                         |
| `textarea`      | `TEXT`                                 | Multi-line                          |
| `url`           | `VARCHAR(255)`                         | Validated URL format                |
| `slug`          | `VARCHAR(255)`                         |                                     |
| `color`         | `VARCHAR(255)`                         | Hex color                           |
| `ip`            | `INET`                                 |                                     |
| `phone`         | `VARCHAR(20)` or `VARCHAR(n)` if `max_length: n` | `<input type="tel">`  |
| `int`           | `INTEGER`                              |                                     |
| `bigint`        | `BIGINT`                               |                                     |
| `float`         | `DOUBLE`                               |                                     |
| `decimal`       | `DECIMAL`                              |                                     |
| `percent`       | `DOUBLE`                               | Stored as float                     |
| `bool`          | `BOOLEAN`                              |                                     |
| `date`          | `DATE`                                 |                                     |
| `time`          | `TIME`                                 |                                     |
| `datetime`      | `DATETIME`                             |                                     |
| `uuid`          | `UUID`                                 |                                     |
| `json`          | `TEXT`                                 |                                     |
| `image`         | `VARCHAR(255)`                         | Stores file path                    |
| `document`      | `VARCHAR(255)`                         | Stores file path                    |
| `file`          | `VARCHAR(255)`                         | Stores file path                    |
| `choice`        | `VARCHAR` / native `ENUM`              | Requires `enum(EnumName)`           |
| `radio`         | Same as `choice`                       | Different widget, same SQL          |
| `checkbox`      | Same as `choice`                       | Different widget, same SQL          |

---

## Field options — syntax v1

In a `[...]` block, comma-separated:

```rust
username: String [required, max_len(150), unique],
```

| Option              | Description                                                    |
|---------------------|----------------------------------------------------------------|
| `required`          | `NOT NULL` column + form validation                            |
| `nullable`          | `NULL` column — Rust type `Option<T>`                         |
| `unique`            | `UNIQUE` constraint                                            |
| `index`             | Simple index (non-unique)                                      |
| `default(value)`    | SQL default value (`true`, `0`, `"draft"`, etc.)              |
| `max_len(n)`        | Max length (validation + `VARCHAR(n)`)                        |
| `min_len(n)`        | Min length (validation)                                       |
| `max(n)`            | Max integer value (validation)                                |
| `min(n)`            | Min integer value (validation)                                |
| `max_f(n)`          | Max float value                                               |
| `min_f(n)`          | Min float value                                               |
| `auto_now`          | Set to `NOW()` on every `INSERT` — excluded from forms        |
| `auto_now_update`   | Set to `NOW()` on every `UPDATE` — excluded from forms        |
| `readonly`          | Excluded from generated forms                                 |
| `select_as(str)`    | SQL alias in SELECTs                                          |
| `label("str")`      | Custom label in admin forms                                   |
| `help("str")`       | Help text (reserved)                                          |
| `fk(table.col, action)` | Foreign key constraint (see Relations)                   |
| `file(kind)`        | File field — `image`, `document`, `any`                       |
| `file(kind, "path")`| File field with explicit upload directory                     |
| `max_size(n)`       | Max upload size — `n KB`, `n MB`, `n GB`                      |

## Field options — syntax v2

Using `:` instead of `()` for values:

```rust
username: text [required, max_length: 150, unique],
```

| v2 option              | v1 equivalent          | Notes                          |
|------------------------|------------------------|--------------------------------|
| `required`             | `required`             |                                |
| `nullable`             | `nullable`             |                                |
| `unique`               | `unique`               |                                |
| `max_length: n`        | `max_len(n)`           |                                |
| `min_length: n`        | `min_len(n)`           |                                |
| `min: n`               | `min(n)`               |                                |
| `max: n`               | `max(n)`               |                                |
| `min: n.0`             | `min_f(n)`             |                                |
| `max: n.0`             | `max_f(n)`             |                                |
| `default: value`       | `default(value)`       |                                |
| `auto_now`             | `auto_now`             |                                |
| `auto_now_update`      | `auto_now_update`      |                                |
| `upload_to: "path"`    | `file(kind, "path")`   |                                |
| `max_size: n MB`       | `max_size(n MB)`       |                                |
| `rows: n`              | —                      | v2 only (textarea)             |
| `step: n`              | —                      | v2 only (numeric fields)       |
| `fk(table.col, action)`| `fk(table.col, action)`|                                |
| `enum(EnumName)`       | `enum(EnumName)`       |                                |
| `renamed_from: "x"`    | `renamed_from("x")`    | Renames a column (see below)   |
| `skip`                 | `readonly`             |                                |
| `no_hash`              | —                      | `password` fields only         |

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

### Four variant forms

| Syntax                               | DB value         | Display label (Display)   |
|--------------------------------------|------------------|---------------------------|
| `Variant`                            | `"Variant"`      | `"Variant"`               |
| `Variant: "Label"`                   | `"Variant"`      | `"Label"`                 |
| `Variant = "db_value"`               | `"db_value"`     | `"db_value"`              |
| `Variant = ("db_value", "Label")`    | `"db_value"`     | `"Label"`                 |

> **The DB value is stored exactly as written.** No automatic transformation.

### Backing types

| Syntax               | DB storage                                      |
|----------------------|-------------------------------------------------|
| `EnumName: [A, B]`   | Native `ENUM` (Postgres) or `VARCHAR` (MySQL/SQLite) |
| `EnumName: i32 [...]`| `INTEGER`                                       |
| `EnumName: i64 [...]`| `BIGINT`                                        |

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
|----------------|-----------------|-------------------------------|
| `belongs_to`   | ❌ code only     | N-1 relation (SeaORM)         |
| `has_many`     | ❌ code only     | 1-N relation                  |
| `has_one`      | ❌ code only     | 1-1 relation                  |
| `many_to_many` | ❌ code only     | N-N via pivot table           |

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
|-----------------------|-----------------------|---------------------------------------------|
| `ordering`            | `[field, -field]`     | Default sort order, `-` = `DESC`            |
| `unique_together`     | `[(col1, col2)]`      | Multi-column `UNIQUE` constraint            |
| `indexes`             | `[(col1, col2)]`      | Multi-column simple index                   |
| `verbose_name`        | `"string"`            | Singular name in the admin interface        |
| `verbose_name_plural` | `"string"`            | Plural name in the admin interface          |
| `abstract`            | `true`                | Abstract model — no table generated         |

---

## `label` and `help`

By default, the label is generated from the snake_case field name (`sort_order` → `Sort order`). The `label(...)` option overrides it:

```rust
fields: {
    title:        text [required, label("Article title")],
    sort_order:   i32  [label("Display order")],
    is_published: bool [label("Published")],
},
```

> `label` and `help` are **v1 only** options — not available in the v2 anonymous block.

The label applies to the admin form and column headers in `list_display`. It has no effect on migrations.

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

Allowed tables: `eihwaz_users`, `eihwaz_groupes`, `eihwaz_droits`, `eihwaz_sessions`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`. Any other name causes a compile-time error.

Fields in `extend!{}` use the same types and options as the v2 syntax of `model!` (including `renamed_from`). No `relations:` block inside `extend!{}` — relations are declared in the target `model!{}` with `has_many(user_profile)` etc.

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
