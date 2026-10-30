# RuniqueForm trait

[← Form extraction](/docs/en/formulaire/prisme)

---

## Base structure

Each form contains a `form: Forms` field. Field registration and accessors live on **`FormAccess`**; business logic (`clean`, hooks…) lives on **`RuniqueForm`**:

```rust
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct UsernameForm {
    pub form: Forms,
}

impl FormAccess for UsernameForm {
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

impl RuniqueForm for UsernameForm {}
```

> **💡 Split:** `FormAccess` owns `register_fields` + accessors; `RuniqueForm` owns validation hooks. `impl_form_access!()` generates `from_form()`, `get_form()` and `get_form_mut()` inside `impl FormAccess`. If your field is not named `form`, pass the name: `impl_form_access!(formulaire)`.
>
> With `#[form(schema = …)]`, the macro generates `impl FormAccess` for you — user code only writes `impl RuniqueForm` (`clean`, `customize`…).
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
FormAccess::register_fields() → declare fields
        ↓
req.form()                   → build the instance
        ↓
is_valid()                   → validation pipeline
    ↓  validate() per field  (required, format, length…)
    ↓  clean_field(name)     [optional — per-field business rule]
    ↓  clean()               [optional — cross-field validation]
    ↓  finalize()            (Argon2 hashing, final transforms)
        ↓
form.save(&db) / delete(&db)  → write, on the validated form
        ↓
database_error()             → [optional] DB error handling
        ↓
clear()                      → [optional] empty the form after processing
```

**Method reference:**

**`register_fields(form)`** (`FormAccess`) — Declare the form fields.

**`from_form(form)`** (`FormAccess`) — Build the instance from a `Forms`.

**`get_form()` / `get_form_mut()`** (`FormAccess`) — Low-level accessors for the internal `Forms` (useful for rendering or dynamic field configuration). Do not use for data extraction.

**`clean_field(name)`** *(optional)* — Per-field business validation. Returns `bool`. Called after `validate()` for each field.

**`clean()`** *(optional)* — Cross-field validation. Returns `Result<(), StrMap>`. Called once all fields are valid.

**`is_valid()`** — Orchestrates the full pipeline. Safe to call on both GET and POST: returns `false` without setting field errors if no data has been submitted (first page load), validates normally otherwise.

**`save(&db)` / `delete(&db)`** — On the validated form `try_form` returns, for a form tied to an entity: writes the row, returns `Ok(form)` or `Err(form)`. See "Writing" below.

**`database_error(&err)`** — Parses a DB error and attaches it to the correct field. `ValidationForm::save` already calls it: the `Err(form)` comes back with the error. Call it yourself only after a hand-written save that returns a `DbErr`.

**`clear()`** — Clears all field values (except CSRF and the form's name) and resets `submitted` to `false`. Call it after reading cleaned data, before a redirect or empty re-render.

---

## Accessing data — `cleaned_*()`

After `is_valid()`, access data via the `cleaned_*` methods on the trait:

```rust
self.cleaned::<String>("username")      // Option<String>  (None if empty)
self.cleaned::<i32>("age")              // Option<i32>
self.cleaned::<f64>("price")            // Option<f64>  (handles , → .)
self.cleaned::<bool>("active")          // Option<bool>
// … see the Helpers page for the full list
```

These methods are **whitelisted**: they return `None` if the field is not declared in `register_fields()`. They also cover route parameters and query string parameters.

> **`cleaned::<String>` returns `Option<String>`**. For an empty string default: `self.cleaned::<String>("x").unwrap_or_default()`.

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

## Form name — several forms on one page

`try_form(form, &request, "name")` names the form. The name is rendered as a hidden `runique_form` field (first, before CSRF) and prefixes the fields' HTML ids (`id_name-field`). On POST, the form is only validated when the request sends **that same name** back: on a page showing several forms, only the one submitted is validated, the others come back as `Err` with no error set.

- Every handler that **renders** the form goes through `try_form`, GET included: a form never named fails to render (Tera error on the missing name), rather than showing a form whose submission would never validate.
- The name is never filled from the request: only `try_form` sets it.
- A hand-written form (without `{% form.x %}`) must carry the hidden `runique_form` field itself, with the right name.

```rust
let username_form = ValidationForm::try_form(request.form::<UsernameForm>(), &request, "username").await;
let email_form = ValidationForm::try_form(request.form::<EmailForm>(), &request, "email").await;
```

---

## `is_valid()` — calling on GET and POST

`is_valid()` validates regardless of the HTTP method, but it **never looks at the method itself** — that's `ValidationForm::try_form(form, &request, "my_form")`'s job, via `allow_get`/`allow_post`, deciding whether validation should even be attempted. `allow_get` defaults to `false` (CSRF protection: GET is the only method exempt from CSRF, and a mutating form must never auto-validate on it). A read-only search form is the case that must **explicitly** lift that guard:

- **First GET (empty form)** — `allow_get` falls back to `is_submitted()` → `false`, `try_form` returns `Err` without attempting validation, no errors set. The template renders a clean empty form.
- **GET with query params (search form)** — `is_submitted()` → `true`, `try_form` validates and returns `Ok`. Enables GET-based searches without extra code.
- **POST** — standard behavior: validates and sets field errors if invalid (also needs an `allow_post` that permits it — the default already covers this case).

```rust
impl RuniqueForm for SearchForm {
    // ...
    fn allow_get(&self, _request: &Request) -> bool {
        self.is_submitted()
    }
}

// Unified GET+POST handler — no method branching needed
pub async fn search(mut request: Request) -> AppResult<Response> {
    let form: SearchForm = request.form();
    let form = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => {
            let query = validated.cleaned::<String>("q").unwrap_or_default();
            // run the search...
            validated.into_form()
        }
        // First GET or invalid GET/POST: no errors if nothing was
        // submitted, field errors set otherwise.
        Err(form) => form,
    };
    context_update!(request => { "search_form" => &form });
    request.render("search.html")
}
```

> To explicitly distinguish "first page load" from "form submitted with invalid data", use `request.method.is_safe()` (GET/HEAD).

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
            let val = self.cleaned::<String>("username").unwrap_or_default();
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
        let pwd1 = self.cleaned::<String>("password").unwrap_or_default();
        let pwd2 = self.cleaned::<String>("password_confirm").unwrap_or_default();

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

## Writing — `save` and `delete`

A form tied to an entity (`model!{}`, `extend!{}`, `#[form(schema = …)]`) is written from the validated form `ValidationForm::try_form` returns. Runique walks its fields and writes the row: there is no write method to implement. A form without an entity has neither `save` nor `delete`: its handler does the work after `try_form`.

```rust
match ValidationForm::try_form(form, &request, "article").await {
    Ok(form) => match form.save(&request.engine.db).await {
        Ok(_) => {
            success!(request.notices => "Article saved");
            Ok(Redirect::to("/articles").into_response())
        }
        Err(form) => {
            context_update!(request => { "article_form" => &form });
            request.render(template)
        }
    },
    Err(form) => {
        context_update!(request => { "article_form" => &form });
        request.render(template)
    }
}
```

`save` and `delete` give the form back either way: `Ok(form)` once written (redirect, or `clear()` it and render again), `Err(form)` with the reason on its fields or its errors.

### The row comes from the URL

The key is looked up in the route's parameters, by the name given to `try_form`:

1. `{<name>_<key>}` — e.g. `{article_id}` for the form `"article"`;
2. else `{<key>}` — e.g. `{id}`;
3. else none: `save` creates the row.

With a key, `save` updates that row; `delete` needs one. A key that doesn't read as the model's key type, or a missing row, gives "Resource not found" and nothing is written. Two forms on one page each point at their row by their own name.

### What is written

- **Outside the admin**: SeaORM writes (`insert` on a create, `ActiveModel::save()` on an update); the model's behavior (`ActiveModelBehavior::before_save` / `after_save`, `auto_now`) runs. Locked fields (`readonly`, `disabled`) and secret columns (password) are never written. Checking that the user may touch the row is the handler's job, as with `search!`.
- **Inside the admin** (panel and routes declared `admin`): the gate writes, as the checked account — rights, owner, history, in one transaction.

### Where business logic goes

- Values to check or adjust: `clean_field`, `clean`.
- Values from the request (signed-in account, choices): `register_dynamic_fields`.
- Outside effects (email, cache) and writes to other tables: in the handler, after a successful `save`.

---

## `clear()` — empty the form after processing

`clear()` resets all field values (except the CSRF token) and sets `submitted` back to `false`.

Available anywhere `self` is `&mut Self` — from a handler or from a method on the form itself.

### From a handler

```rust
if form.is_valid().await {
    let path = form.cleaned::<String>("image").unwrap_or_default(); // 1. read before clear
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
            title: Set(self.cleaned::<String>("title").unwrap_or_default()),
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
