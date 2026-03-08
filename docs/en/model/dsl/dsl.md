# `model!` DSL & AST

## Actually exposed macros

In the `derive_form` crate, the available macros are:

- `model!(...)` (proc macro)
- `#[form(...)]` (attribute proc macro)

On the Runique API side (`prelude`), `model` and `form` are re-exported.

---

## `model!(...)` DSL: expected structure

The parser expects a strict structure:

1. model name,
2. `table: "..."`,
3. `pk: id => i32|i64|uuid`,
4. `fields: { ... }`,
5. `relations: { ... }` (optional),
6. `meta: { ... }` (optional).

Concrete example:

```rust
use runique::prelude::*;

model!(
    User,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email: String [required, unique],
        password: String [required],
        is_active: bool [default(true)],
        created_at: datetime [auto_now],
    },
    relations: {
        has_many(Post),
    },
    meta: {
        ordering: [-created_at],
    }
);
```

---

## Internal AST (what is parsed)

The DSL is converted into an internal `Model` AST structure, including:

- `name`, `table`, `pk`
- `fields: Vec<FieldDef>`
- `relations: Vec<RelationDef>`
- `meta: Option<MetaDef>`

### Supported types

- text: `String`, `text`, `char`, `varchar(n)`
- numeric: `i8/i16/i32/i64/u32/u64/f32/f64`, `decimal(p,s)`
- date/time: `date`, `time`, `datetime`, `timestamp`, `timestamp_tz`
- other: `bool`, `uuid`, `json`, `json_binary`, `binary`, `blob`, `enum(...)`, `inet`, `cidr`

### Field options

- `required`, `nullable`, `unique`, `index`
- `default(...)`, `max_len(...)`, `min_len(...)`, `max(...)`, `min(...)`
- `auto_now`, `auto_now_update`, `readonly`
- `fk(table.column, cascade|set_null|restrict|set_default)`

---

## See also

| Section | Description |
| --- | --- |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code, `ModelSchema` |
| [Forms & technical considerations](https://github.com/seb-alliot/runique/blob/main/docs/en/model/forms/forms.md) | `#[form(...)]` |

## Back to summary

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
