🌍 **Languages**: [English](/runique/runique_dsl/README.md) | [Français](/runique/runique_dsl/README.fr.md)

# runique_dsl

Reads the [Runique](https://runique.io/) model DSL — `model!{}` and `extend!{}` — into an AST,
without generating any code.

Two consumers read the DSL through this crate:

- the `derive_form` macros, which generate the SeaORM entity, the forms and the schema;
- `runique makemigrations`, which reads the same declarations from your source files to write
  the SQL migrations.

Because both use one parser, a model the macro refuses is refused by the CLI too, at the same
place and with the same message (file, line, column). Nothing is skipped silently.

You don't depend on this crate directly: `runique` and `derive_form` do. Publication order is
`runique_dsl` → `derive_form` → `runique`.

## The DSL

```rust
model! {
    Book,
    table: "books",
    pk: id => Pk,
    enums: {
        Status: [Draft, Published = ("published", "Published")],
        Priority: i32 [Low = 1, High = (2, "High")],
    },
    {
        title:      text [required, max_length: 200],
        summary:    textarea [nullable],
        status:     choice [enum(Status), required],
        priority:   choice [enum(Priority), required],
        price:      decimal [required, min: 0.0],
        shelf_code: int [nullable],
        cover:      image [nullable, upload_to: "covers/"],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    },
    relations: {
        belongs_to: shelf via shelf_code [set_null, cascade],
    },
    meta: {
        ordering: [-created_at],
        unique_together: [(title, shelf_code)],
        indexes: [(status)],
        verbose_name: "Book",
    }
}
```

`extend!{}` adds fields to a framework table, with the same field and enum grammar:

```rust
extend! {
    table: "eihwaz_users",
    fields: {
        bio:     textarea [nullable],
        website: url [nullable],
    }
}
```

### Header

- `table: "name"` — SQL table name, validated as an identifier.
- `pk: name => i32 | i64 | uuid | Pk` — `Pk` follows the `big-pk` / `pk-uuid` features
  (`i32` by default).

### Enums

- `Name: [A, B]` — stored as the variant name (a native enum on Postgres, `VARCHAR` elsewhere).
- `A = "value"`, `A: "label"`, `A = ("value", "label")` — stored value and/or displayed label.
- `Name: i32 [...]` / `Name: i64 [...]` — stored as a number. **Every variant needs an integer
  value** (`Low = 1` or `Low = (1, "label")`), within the type's range, and no two variants may
  share one.

### Field attributes

| Attribute | Effect |
|---|---|
| `required` | The form field is mandatory. Says nothing about the column. |
| `nullable` | The column accepts NULL and the Rust field is `Option<T>`. |
| `default: lit` | SQL default; the literal must match the type. |
| `unique` | UNIQUE constraint. |
| `max_length: n` / `min_length: n` | Text length (and byte length for `binary`/`var_binary`). `max_length` sets `VARCHAR(n)`. |
| `min: n` / `max: n` | Integer or float bounds, checked by the forms. |
| `auto_now` / `auto_now_update` | Set on insert / on every save; not in the forms. |
| `enum(Name)` | The enum a `choice` / `radio` / `checkbox` draws from. |
| `upload_to: "dir"`, `max_size: 5MB` | File fields (`upload_to` is required on them). |
| `label: "…"`, `rows: n`, `step: x`, `no_hash` | Form rendering details. |
| `readonly` | Kept out of migrations and forms. |
| `skip` | In the SQL schema, kept out of the generated forms. |
| `renamed_from: "old"` | Migration only: rename the column instead of drop + add. |

### Nullability

A column is **NOT NULL unless declared `nullable`**. The parser refuses, at compile time:

- `required` together with `nullable`;
- `nullable` on `auto_now` / `auto_now_update` (the framework always fills them);
- a field that isn't text-like (number, date, choice, uuid, json…) declared without
  `required`, `nullable` or `default` — an empty optional field would have nothing to store.
  Text, file, binary and `bool` fields can stay optional: they always submit a storable value.

### Relations

- `belongs_to: target via column [on_delete, on_update]` — the **only** way to declare a foreign
  key. It produces the SQL constraint and the SeaORM relation. `target` is the entity module
  (the file `target.rs`) or a framework table (`eihwaz_users`, …); the key references that
  model's real table and primary key. Actions: `cascade`, `set_null`, `restrict`,
  `set_default`, `no_action` (default). `column` must be a declared field, and `nullable` when
  an action is `set_null`.
- `has_many: target` and `has_one: target`, optionally followed by `as name`.
- `many_to_many: target through junction via column`.

## Types

`runique_dsl::types` holds, for each DSL type, its column, its form field and its bounds.

| DSL type | Column | Form field |
|---|---|---|
| `text`, `char` | `VARCHAR` (`VARCHAR(n)` with `max_length`) | text |
| `email` | `VARCHAR(254)` | email |
| `phone` | `VARCHAR(20)` (or `max_length`) | phone |
| `password`, `url`, `slug`, `color` | `VARCHAR` | password (hashed) / url / slug / color |
| `textarea`, `richtext` | `TEXT` | textarea / rich text (sanitized) |
| `image`, `document`, `file` | `VARCHAR` (stored path) | file upload |
| `int`, `bigint`, `i8`, `i16`, `u32`, `u64` | `INTEGER`, `BIGINT`, `TINYINT`, `SMALLINT`, unsigned | integer, held to the Rust type's range |
| `float`, `f32`, `percent`, `decimal` | `DOUBLE`, `FLOAT`, `DOUBLE`, `DECIMAL` | number (`percent`: 0–100) |
| `bool` | `BOOLEAN` | checkbox |
| `date`, `time`, `datetime`, `timestamp`, `timestamp_tz` | `DATE`, `TIME`, `DATETIME`, `DATETIME`, `TIMESTAMP WITH TIME ZONE` | date / time pickers |
| `uuid` | `UUID` | uuid |
| `json`, `json_binary` | `JSON` | json |
| `binary`, `var_binary`, `blob` | `BINARY(n)`, `VARBINARY(n)` (255 by default), `BLOB` | bytes upload |
| `ip`, `cidr`, `mac_address`, `interval` | `VARCHAR` | ip / text |
| `choice`, `radio`, `checkbox` + `enum(X)` | the enum's column | select / radio / checkboxes |
| `Pk` | same type as the primary keys | integer or uuid |

Engine limits, refused at compile time when a single engine feature is enabled: `i8` and
`u32` on Postgres, `u64` on Postgres and SQLite. On MySQL/MariaDB, `binary` is a fixed-length
column padded with `0x00`: use `var_binary` to get the exact bytes back.

## Breaking changes in 3.0

| Before | Now |
|---|---|
| A field without `required` was nullable. | NOT NULL unless `nullable`; `required` only makes the form field mandatory. Add `nullable` to keep an existing optional column — `makemigrations` stops with `nullable -> not_null` otherwise. |
| `auto_now` / `auto_now_update` fields were `Option<T>`. | Plain `T`, always set by the framework. |
| Foreign keys through `fk(table.column, action)` on the field. | `relations: { belongs_to: target via column [on_delete, on_update] }`. `fk(...)` is an error pointing there. |
| `makemigrations` skipped a model it couldn't read, and kept its own copy of the grammar (v1 types `String`, `i32`…). | One strict parser: unknown type or attribute, unknown FK action, two `model!{}` in one file, unresolved `belongs_to` target → error with file, line and column. |
| `i32` / `i64` enum variants without a value were stored as `0`. | Every variant needs a unique integer value within the type's range. |
| `customize` loosening a declared `max_length` was silently capped. | Loosening `min_length`, `max_length`, `min` or `max` panics when the form is built; tightening is allowed. |
| `runique::migration::RelationDef` / `RelationKind`, `ModelSchema::relation()`. | Removed (never read). Relations live on the SeaORM entity. |

Also new, without breaking anything: `max_length` reaches the migrations (`VARCHAR(n)`,
`BINARY(n)`, `VARBINARY(n)`), followed by the snapshots and the diff — growing a column is a
plain `ALTER`, shrinking it needs `--force`. A snapshot written before 3.0 takes the model's
lengths once, so upgrading produces no migration of its own. `blob` gets its own column type,
and `belongs_to` targets the real primary key of the related entity, with its actions.

## License

MIT
