# `model!` DSL & AST

## Actually exposed macros

In the `derive_form` crate, the available macros are:

- `model! { ... }` (proc macro)
- `#[form(...)]` (attribute proc macro)

On the Runique API side (`prelude`), `model` and `form` are re-exported.

---

## `model! { ... }` DSL: expected structure

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

model! {
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
        has_many: Post,
        belongs_to: Team via team_id,
    },
    meta: {
        ordering: [-created_at],
        verbose_name: "user",
    }
}
```

---

## Internal AST (what is parsed)

The DSL is converted into an internal `Model` AST structure, including:

- `name`, `table`, `pk`
- `fields: Vec<FieldDef>`
- `relations: Vec<RelationDef>`
- `meta: Option<MetaDef>`

### Supported types

- text: `String`, `text`, `char`, `varchar(n)`, `var_binary(n)`
- numeric: `i8/i16/i32/i64/u32/u64/f32/f64`, `decimal(p,s)`, `decimal`
- date/time: `date`, `time`, `datetime`, `timestamp`, `timestamp_tz`, `interval`
- other: `bool`, `uuid`, `json`, `json_binary`, `binary(n)`, `binary`, `blob`, `enum(A, B, ...)`, `inet`, `cidr`, `mac_address`

### Field options

- `required`, `nullable`, `unique`, `index`, `readonly`
- `default(...)`, `max_len(n)`, `min_len(n)`, `max(n)`, `min(n)`, `max_f(n)`, `min_f(n)`
- `auto_now`, `auto_now_update`
- `label("...")`, `help("...")`, `select_as("...")`
- `fk(table.column, cascade|set_null|restrict|set_default)`

### Relations

| Syntax | Description |
| --- | --- |
| `has_many: Model,` | 1-N relation |
| `has_many: Model as alias,` | 1-N with custom accessor name |
| `has_one: Model,` | 1-1 relation |
| `has_one: Model as alias,` | 1-1 with custom accessor name |
| `belongs_to: Model via fk_field,` | Incoming foreign key |
| `many_to_many: Model through pivot_table,` | N-N relation |

### Meta

| Key | Value | Description |
| --- | --- | --- |
| `ordering: [field, -field]` | list | Default sort order (`-` = DESC) |
| `unique_together: [(f1, f2), ...]` | list of tuples | Composite uniqueness constraint |
| `verbose_name: "..."` | string | Display name (singular) |
| `verbose_name_plural: "..."` | string | Display name (plural) |
| `abstract: true` | bool | Abstract model (no table) |
| `indexes: [(f1, f2), ...]` | list of tuples | Composite indexes |

---

## See also

| Section | Description |
| --- | --- |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code, `ModelSchema` |
| [Forms & technical considerations](https://github.com/seb-alliot/runique/blob/main/docs/en/model/forms/forms.md) | `#[form(...)]` |

## Back to summary

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
