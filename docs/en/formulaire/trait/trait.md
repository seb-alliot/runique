# RuniqueForm trait

[← Form extraction](/docs/en/formulaire/prisme)

---

## Base structure

Each form contains a `form: Forms` field and implements the `RuniqueForm` trait:

```rust
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UsernameForm {
    pub form: Forms,
}

impl RuniqueForm for UsernameForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Username")
                .required()
                .placeholder("Enter a username"),
        );
    }

    impl_form_access!();
}
```

> **💡 `impl_form_access!()`** automatically generates `from_form()`, `get_form()` and `get_form_mut()`. If your field is not named `form`, pass the name as an argument: `impl_form_access!(formulaire)`.
>
> **Security note:** `get_form()` provides access to the internal structure. For extracting cleaned data, always use the `cleaned_*` methods of the trait (see below). Direct extraction methods on `Forms` are reserved for internal framework use.

<details>
<summary>Equivalent without the macro (for reference)</summary>

```rust
fn from_form(form: Forms) -> Self {
    Self { form }
}
fn get_form(&self) -> &Forms {
    &self.form
}
fn get_form_mut(&mut self) -> &mut Forms {
    &mut self.form
}
```

</details>

---

## RuniqueForm trait methods

**Form lifecycle (call order):**

```text
register_fields()            → declare fields
        ↓
build() / build_with_data()  → build the instance
        ↓
is_valid()                   → validation pipeline
    ↓  validate() per field  (required, format, length…)
    ↓  clean_field(name)     [optional — per-field business rule]
    ↓  clean()               [optional — cross-field validation]
    ↓  finalize()            (Argon2 hashing, final transforms)
        ↓
save() / save_as()           → persistence (with or without hooks)
  ↓  before_save()           [optional — hook before persistence]
  ↓  on_save()               → main logic (INSERT, UPDATE…)
  ↓  after_save()            [optional — hook after persistence]
        ↓
database_error()             → [optional] DB error handling
        ↓
clear()                      → [optional] empty the form after processing
```

**Method reference:**

**`register_fields(form)`** — Declare the form fields.

**`from_form(form)`** — Build the instance from a `Forms`.

**`get_form()` / `get_form_mut()`** — Low-level accessors for the internal `Forms` (useful for rendering or dynamic field configuration). Do not use for data extraction.

**`clean_field(name)`** *(optional)* — Per-field business validation. Returns `bool`. Called after `validate()` for each field.

**`clean()`** *(optional)* — Cross-field validation. Returns `Result<(), StrMap>`. Called once all fields are valid.

**`is_valid()`** — Orchestrates the full pipeline. Safe to call on both GET and POST: returns `false` without setting field errors if no data has been submitted (first page load), validates normally otherwise.

**`on_save(&txn)`** *(optional)* — Main persistence logic, executed inside the transaction. This is where DB writes go (INSERT, UPDATE, DELETE). No-op by default.

**`before_save(ctx, &txn)`** *(optional)* — Hook executed inside the transaction before `on_save`. No-op by default.

**`after_save(ctx, &txn)`** *(optional)* — Hook executed inside the transaction after `on_save`. No-op by default.

**`save(&db)`** — Opens a transaction, calls `on_save`, commits. Automatic rollback on error.

**`save_as(ctx, &db)`** — Like `save`, but orchestrates all three hooks: `before_save` → `on_save` → `after_save`. Rollback on any error.

**`database_error(&err)`** — Parses a DB error and attaches it to the correct field.

**`clear()`** — Clears all field values (except CSRF) and resets `submitted` to `false`. Call it after reading cleaned data, before a redirect or empty re-render.

**`build(tera, csrf_token)`** — Build an empty form.

**`build_with_data(data, tera, csrf)`** — Build, fill, and validate.

---

## Accessing data — `cleaned_*()`

After `is_valid()`, access data via the `cleaned_*` methods on the trait:

```rust
self.cleaned_string("username")      // Option<String>  (None if empty)
self.cleaned_i32("age")              // Option<i32>
self.cleaned_f64("price")            // Option<f64>  (handles , → .)
self.cleaned_bool("active")          // Option<bool>
// … see the Helpers page for the full list
```

These methods are **whitelisted**: they return `None` if the field is not declared in `register_fields()`. They also cover route parameters and query string parameters.

> **`cleaned_string` returns `Option<String>`**. For an empty string default: `self.cleaned_string("x").unwrap_or_default()`.

---

## Field overrides — display and constraints

These methods let you modify a field's behaviour **after** `register_fields()`, without changing its declaration. They are chainable and available directly on the `RuniqueForm` trait.

### Display

```rust
form.label("title", "Article title")
    .placeholder("title", "e.g. My article")
    .required("title", false)       // relax the required constraint
    .readonly("title", true)
    .disabled("draft", true)
    .attr("photo", "accept", "image/*");  // arbitrary HTML attribute
```

### Max size on a `FileField`

The model can declare an immutable ceiling via `.max_size()` in `register_fields()`. A form-level override cannot exceed it:

```rust
// In register_fields — model ceiling set to 5 MB
form.field(&FileField::image("photo").max_size(FileSize::mb(5)));

// In the handler — override allowed if ≤ ceiling
form.max_size("photo", FileSize::mb(3))?;   // Ok
form.max_size("photo", FileSize::mb(6))?;   // Err — exceeds model ceiling
```

> **Separation of concerns**: the model owns the maximum constraint, the form can narrow it down. Same principle as Django's `ModelForm` for `max_length`.

---

## `is_valid()` — calling on GET and POST

`is_valid()` is designed to be called regardless of the HTTP method:

- **First GET (empty form)** — returns `false`, no errors set on fields. The template renders a clean empty form.
- **GET with query params (search form)** — validates normally, enabling GET-based searches without extra code.
- **POST** — standard behavior: validates and sets field errors if invalid.

```rust
// Unified GET+POST handler — no method branching needed
pub async fn search(mut request: Request) -> AppResult<Response> {
    let mut form: SearchForm = request.form();
    if form.is_valid().await {
        let query = form.cleaned_string("q").unwrap_or_default();
        // run the search...
    }
    // First GET: is_valid() == false, no errors → clean empty form
    // Submitted GET invalid: is_valid() == false, errors shown
    context_update!(request => { "search_form" => &form });
    request.render("search.html")
}
```

> To explicitly distinguish "first page load" from "form submitted with invalid data", use `request.is_post()`.

---

## `is_valid()` validation pipeline

Calling `form.is_valid().await` triggers **4 steps in order** (only if the form is submitted):

1. **Field validation** — Each field runs `validate()`: required, length, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`, …)
2. **`clean_field(name)`** — Per-field business validation, called for each field after step 1 (only if standard validation passed)
3. **`clean()`** — Cross-field validation (e.g. `pwd1 == pwd2`); passwords are still plain text at this step
4. **`finalize()`** — Final transformations (automatic hashing for `Password` fields if the global config is in `Auto` mode)

---

## `clean_field` — per-field business validation

`clean_field` is called for each field after its standard validation. Use it to implement a business rule on a specific field (reserved value, custom format, lightweight uniqueness check…).

- Returns `true` if the field is valid, `false` otherwise
- On failure, set the error manually on the field via `set_error()`
- **Not called** if the required field is already empty (standard validation fails first)

```rust
#[async_trait::async_trait]
impl RuniqueForm for UsernameForm {
    // ...

    async fn clean_field(&mut self, name: &str) -> bool {
        if name == "username" {
            let val = self.cleaned_string("username").unwrap_or_default();
            if val.to_lowercase().contains("admin") {
                if let Some(f) = self.get_form_mut().fields.get_mut("username") {
                    f.set_error("The name 'admin' is reserved".to_string());
                }
                return false;
            }
        }
        true
    }
}
```

> **💡** `clean_field` is ideal for isolated rules on a single field. For rules that involve multiple fields at once, use `clean()`.
>
> **⚠️ Do not call `clean_field` from within `clean`**: the pipeline guarantees that `clean_field` has already run for every field before `clean` is called. Calling it again would be redundant and could set a duplicate error on a field. Furthermore, `clean` is only invoked if all `clean_field` calls returned `true` — so from within `clean`, all fields are already individually valid.

---

## `clean` — cross-field validation

`clean` is called once **all** fields have passed validation (standard + `clean_field`). Use it to cross-check values across fields.

- Returns `Ok(())` if the form is valid
- Returns `Err(StrMap)` with a map `{ "field_name" => "error message" }` on failure

```rust
#[async_trait::async_trait]
impl RuniqueForm for RegisterForm {
    // ...

    async fn clean(&mut self) -> Result<(), StrMap> {
        let pwd1 = self.cleaned_string("password").unwrap_or_default();
        let pwd2 = self.cleaned_string("password_confirm").unwrap_or_default();

        if pwd1 != pwd2 {
            let mut errors = StrMap::new();
            errors.insert(
                "password_confirm".to_string(),
                "Passwords do not match".to_string(),
            );
            return Err(errors);
        }
        Ok(())
    }
}
```

> **⚠️ Important**: `Password` fields are hashed during `finalize()` only if the global config is `PasswordConfig::Auto` (the default mode) **and** `.no_hash()` was not applied to the field. In `Manual`, `Delegated`, or `Custom` mode, `finalize()` does not hash — the application is responsible.
> Use `clean()` for any password comparison — it is the only step where they are still readable.
>
> **Login form**: the password field must **not** be hashed — it will be compared against the stored hash. Disable automatic hashing with `.no_hash()`: `TextField::password("password").no_hash().required()`

---

## Save hooks — `save_as`, `on_save`, `before_save`, `after_save`

Two methods are available to persist form data:

| Method | Description |
| --- | --- |
| `save(&db)` | Simple transaction: `on_save` → commit |
| `save_as(ctx, &db)` | Transaction with hooks: `before_save` → `on_save` → `after_save` → commit |

All three hooks run inside the same transaction. If any of them returns `Err`, the transaction is rolled back and the error is propagated — nothing is written to the DB.

### `SaveContext` — distinguishing Create, Update, Delete

```rust
use runique::prelude::SaveContext;

// In the handler:
form.save_as(SaveContext::Create, &db).await?;
form.save_as(SaveContext::Update, &db).await?;
form.save_as(SaveContext::Delete, &db).await?;
```

### Full implementation example

```rust
use async_trait::async_trait;
use runique::prelude::*;
use sea_orm::{DatabaseTransaction, DbErr, Set};

#[async_trait]
impl RuniqueForm for ArticleForm {
    // ...

    async fn before_save(&mut self, ctx: SaveContext, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        if ctx == SaveContext::Create {
            // e.g. check a quota limit, log intent…
        }
        Ok(())
    }

    async fn on_save(&mut self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        let title = self.cleaned_string("title").unwrap_or_default();
        article::ActiveModel {
            title: Set(title),
            ..Default::default()
        }
        .insert(txn)
        .await?;
        Ok(())
    }

    async fn after_save(&mut self, ctx: SaveContext, _txn: &DatabaseTransaction) -> Result<(), DbErr> {
        if ctx == SaveContext::Create {
            // e.g. send a notification, invalidate a cache…
        }
        Ok(())
    }
}
```

> **Note:** `before_save` and `after_save` also receive the `&DatabaseTransaction` — they can perform their own DB reads or writes within the same transaction.
>
> **`save()` without hooks:** if you don't need `before_save`/`after_save`, use `save()` directly — it calls `on_save` without the hook overhead.

---

## `clear()` — empty the form after processing

`clear()` resets all field values (except the CSRF token) and sets `submitted` back to `false`.

Available anywhere `self` is `&mut Self` — from a handler or from a method on the form itself.

### From a handler

```rust
if form.is_valid().await {
    let path = form.cleaned_string("image").unwrap_or_default(); // 1. read before clear
    // save to DB...
    form.clear();                            // 2. empty
    success!(request.notices => "File uploaded!");
    context_update!(request => { "image_form" => &form });
    return request.render(template);         // 3. re-render with empty form
}
```

### From the form itself (`save(&mut self)`)

Declaring `save` with `&mut self` lets you encapsulate the clear inside — the handler does nothing extra:

```rust
impl BlogForm {
    pub async fn save(
        &mut self,
        db: &DatabaseConnection,
    ) -> Result<blog::Model, DbErr> {
        let record = blog::ActiveModel {
            title: Set(self.cleaned_string("title").unwrap_or_default()),
            // ...
            ..Default::default()
        };
        let result = record.insert(db).await;
        if result.is_ok() {
            self.clear(); // automatically empty after success
        }
        result
    }
}
```

### Where `clear()` cannot be called

- In a `&self` method (immutable) — does not compile
- In `clean()` or `clean_field()` — these run **during** `is_valid()`, before `save()` reads the data; calling `clear()` here would wipe the form before it is saved

> **💡 With redirect (PRG)**: if the handler redirects after success (`Redirect::to(...)`), `clear()` is not needed — the new GET request automatically creates a fresh empty instance.

---

← [**Form extraction**](/docs/en/formulaire/prisme) | [**Typed conversion helpers**](/docs/en/formulaire/helpers) →
