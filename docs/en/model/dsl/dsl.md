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
        is_active: bool,
        team_id: i32 [required],
        created_at: datetime [auto_now],
    },
    relations: {
        has_many: Post,
        belongs_to: Team via team_id,
    },
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

- `required`, `nullable`, `unique`, `readonly`
- `max_len(n)`, `min_len(n)`, `max(n)`, `min(n)`, `max_f(n)`, `min_f(n)`
- `auto_now`, `auto_now_update`
- `label(...)`, `help(...)`, `select_as(...)`
- `file(kind)`, `file(kind, "upload/path")` — file field (see below)

### File fields — `file()`

A `String` field can be declared as a file field using the `file()` option. The auto-generated form (`AdminForm`) will then use a `FileField` instead of a `TextField`.

```rust
model! {
    Article,
    table: "articles",
    pk: id => i32,
    fields: {
        title: String [required],

        // image — explicit directory
        image: String [file(image, "media/articles")],

        // document — auto directory from MEDIA_ROOT + field name
        file: String [file(document)],

        // any file type
        attachment: String [file(any, "media/uploads")],
    },
}
```

**Available kinds:**

| Value | Default allowed extensions | Maps to |
| --- | --- | --- |
| `image` | `jpg jpeg png gif webp avif` | `FileField::image()` |
| `document` | `pdf doc docx txt odt` | `FileField::document()` |
| `any` | no filter | `FileField::any()` |

**Upload path:**

| Syntax | Destination |
| --- | --- |
| `file(image, "media/articles")` | `media/articles/` (exact path) |
| `file(image)` | `{MEDIA_ROOT}/{field_name}/` (reads `MEDIA_ROOT` from `.env`) |

> Invalid files are deleted from disk if validation fails. The destination directory is created automatically on the first valid upload.

### Relations

Relations are declared in an optional `relations: { ... }` block after `fields`.

| Syntax | DB constraint | Description |
| --- | --- | --- |
| `belongs_to: Model via fk_field,` | ✅ `FOREIGN KEY` generated | Foreign key to `model.id` |
| `belongs_to: Model via fk_field [cascade],` | ✅ `ON DELETE CASCADE` | FK with on_delete cascade |
| `belongs_to: Model via fk_field [cascade, restrict],` | ✅ | FK with on_delete + on_update |
| `has_many: Model,` | ❌ (code only) | 1-N relation |
| `has_one: Model,` | ❌ (code only) | 1-1 relation |
| `many_to_many: Model via pivot_table,` | ❌ (code only) | N-N relation |

Available FK actions: `cascade`, `restrict`, `set_null`, `set_default` (default: `no_action`).

> `belongs_to` automatically generates a `FOREIGN KEY` in the migration. The FK column (`fk_field`) must be declared in `fields`.

### Meta

> The `meta` block is reserved for future versions (ordering, verbose_name, etc.). It is parsed without error but currently ignored.

---

## See also

| Section | Description |
| --- | --- |
| [Generation & ModelSchema](https://github.com/seb-alliot/runique/blob/main/docs/en/model/generation/generation.md) | Generated code, `ModelSchema` |
| [Forms & technical considerations](https://github.com/seb-alliot/runique/blob/main/docs/en/model/forms/forms.md) | `#[form(...)]` |

## Back to summary

- [Models](https://github.com/seb-alliot/runique/blob/main/docs/en/model/12-model.md)
