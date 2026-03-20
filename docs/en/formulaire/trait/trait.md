# RuniqueForm trait

[← Prisme extractor](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/prisme/prisme.md)

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
save() / database_error()    → persistence or DB error handling
        ↓
clear()                      → [optional] empty the form after processing
```

**Method reference:**

**`register_fields(form)`** — Declare the form fields.

**`from_form(form)`** — Build the instance from a `Forms`.

**`get_form()` / `get_form_mut()`** — Accessors for the internal `Forms`.

**`clean_field(name)`** *(optional)* — Per-field business validation. Returns `bool`. Called after `validate()` for each field.

**`clean()`** *(optional)* — Cross-field validation. Returns `Result<(), StrMap>`. Called once all fields are valid.

**`is_valid()`** — Orchestrates the full pipeline. Safe to call on both GET and POST: returns `false` without setting field errors if no data has been submitted (first page load), validates normally otherwise.

**`is_submitted()`** — Returns `true` if the form received data (POST, or GET with non-empty query params).

**`database_error(&err)`** — Parses a DB error and attaches it to the correct field.

**`clear()`** — Clears all field values (except CSRF) and resets `submitted` to `false`. Call it after reading cleaned data, before a redirect or empty re-render.

**`build(tera, csrf_token)`** — Build an empty form.

**`build_with_data(data, tera, csrf)`** — Build, fill, and validate.

---

## `is_valid()` — calling on GET and POST

`is_valid()` is designed to be called regardless of the HTTP method:

- **First GET (empty form)** — returns `false`, no errors set on fields. The template renders a clean empty form.
- **GET with query params (search form)** — validates normally, enabling GET-based searches without extra code.
- **POST** — standard behavior: validates and sets field errors if invalid.

```rust
// Unified GET+POST handler — no method branching needed
pub async fn search(
    mut request: Request,
    Prisme(mut form): Prisme<SearchForm>,
) -> AppResult<Response> {
    if form.is_valid().await {
        let query = form.get_string("q");
        // run the search...
    }
    // First GET: is_valid() == false, no errors → clean empty form
    // Submitted GET invalid: is_valid() == false, errors shown
    context_update!(request => { "search_form" => &form });
    request.render("search.html")
}
```

> **`is_submitted()`** is available when you need to explicitly distinguish "first page load" from "form submitted with invalid data".

---

## `is_valid()` validation pipeline

Calling `form.is_valid().await` triggers **4 steps in order** (only if the form is submitted):

1. **Field validation** — Each field runs `validate()`: required, length, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`, …)
2. **`clean_field(name)`** — Per-field business validation, called for each field after step 1 (only if standard validation passed)
3. **`clean()`** — Cross-field validation (e.g. `pwd1 == pwd2`); passwords are still plain text at this step
4. **`finalize()`** — Final transformations (automatic Argon2 hashing for `Password` fields)

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
            let val = self.get_form().get_string("username");
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
        let pwd1 = self.form.get_string("password");
        let pwd2 = self.form.get_string("password_confirm");

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

> **⚠️ Important**: `Password` fields are **automatically hashed** during `finalize()` by default (Argon2), unless `password_init` is called in `main.rs` with `PasswordConfig::Manual`, `Delegated`, or `Custom`.
> Use `clean()` for any password comparison — it is the only step where they are still readable.

---

## `clear()` — empty the form after processing

`clear()` resets all field values (except the CSRF token) and sets `submitted` back to `false`.

Available anywhere `self` is `&mut Self` — from a handler or from a method on the form itself.

### From a handler

```rust
if form.is_valid().await {
    let path = form.cleaned_string("image"); // 1. read before clear
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
            title: Set(self.form.get_string("title")),
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

← [**Prisme extractor**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/prisme/prisme.md) | [**Typed conversion helpers**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/helpers/helpers.md) →
