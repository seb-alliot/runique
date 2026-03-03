# derive_form

Procedural macros for the [Runique](https://github.com/seb-alliot/runique) web framework.

Exposes two macros:

- `model!(...)` — DSL to declare a SeaORM model and generate its schema
- `#[form(...)]` — attribute macro to generate a form struct from a model schema

---

## `model!(...)`

Declares a database model and generates the corresponding SeaORM entity, `ActiveModel`, relations, and a `schema()` function used by `#[form(...)]`.

```rust
use derive_form::model;

model! {
    User,
    table: "users",
    pk: id => i32,
    fields: {
        username: String [required, max_len(150), unique],
        email:    String [required, unique],
        password: String [required, max_len(128)],
        is_active: bool  [required],
        created_at: datetime [auto_now],
        updated_at: datetime [auto_now_update],
    }
}
```

### Supported field types

`String`, `i8`, `i16`, `i32`, `i64`, `u32`, `u64`, `f32`, `f64`,
`bool`, `date`, `time`, `datetime`, `timestamp`, `uuid`, `json`, `blob`, `enum(...)`, and more.

### Field options

| Option                        | Description                        |
| ----------------------------- | ---------------------------------- |
| `required` / `nullable`       | NOT NULL vs NULL                   |
| `unique`                      | UNIQUE constraint                  |
| `index`                       | Create an index                    |
| `max_len(n)` / `min_len(n)`   | Length constraints                 |
| `max(n)` / `min(n)`           | Value constraints                  |
| `default(...)`                | Default value                      |
| `auto_now`                    | Set to `now()` on insert           |
| `auto_now_update`             | Set to `now()` on update           |
| `fk(table.col, cascade)`      | Foreign key                        |

---

## `#[form(...)]`

Generates a form struct from the schema produced by `model!`.

```rust
use derive_form::form;

#[form(schema = user_schema, fields = [username, email, password])]
pub struct RegisterForm;
```

### Parameters

| Parameter  | Required | Description                              |
| ---------- | -------- | ---------------------------------------- |
| `schema`   | yes      | Path to the schema function from `model!`|
| `fields`   | no       | Whitelist — only these fields are included |
| `exclude`  | no       | Blacklist — these fields are excluded    |

`fields` and `exclude` are mutually exclusive. The primary key is always excluded.

### What is generated

```rust
#[derive(serde::Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: runique::forms::Forms,
}

impl ModelForm for RegisterForm { ... }
impl RuniqueForm for RegisterForm { ... }
```

The generated struct implements:

- `ModelForm` — `schema()`, `fields()`, `exclude()`
- `RuniqueForm` — `register_fields()`, `from_form()`, `get_form()`, `get_form_mut()`

It can be used directly with Runique's `Prisme<T>` extractor.

---

## Usage with Runique

```rust
use runique::prelude::*;

// 1. Declare the model
model! {
    Post,
    table: "posts",
    pk: id => i32,
    fields: {
        title:   String [required],
        content: String [required],
        slug:    String [required, unique],
    }
}

// 2. Create a form from the schema
#[form(schema = post_schema, fields = [title, content, slug])]
pub struct PostForm;

// 3. Use it in a handler
pub async fn create_post(
    mut req: Request,
    Prisme(mut form): Prisme<PostForm>,
) -> impl IntoResponse {
    if form.is_valid().await {
        form.save(&req.engine.db).await.ok();
        return Redirect::to("/posts").into_response();
    }
    render("posts/new.html", context! { form: form })
}
```

---

## License

MIT — part of the [Runique](https://github.com/seb-alliot/runique) project.
