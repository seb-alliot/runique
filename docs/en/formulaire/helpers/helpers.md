# Typed conversion helpers

[← RuniqueForm trait](/docs/en/formulaire/trait)

---

Form values are stored as `String`. Use the typed helpers to convert them without manual parsing.

## Recommended API — `cleaned_*()`

Methods on the `RuniqueForm` trait. Whitelisted (undeclared field → `None`), covering POST + route parameters + query string.

```rust
self.cleaned_string("username")      // Option<String>  (None if empty)
self.cleaned_i32("age")              // Option<i32>
self.cleaned_i64("count")            // Option<i64>
self.cleaned_u32("quantity")         // Option<u32>
self.cleaned_u64("id")               // Option<u64>
self.cleaned_f32("ratio")            // Option<f32>  (handles , → .)
self.cleaned_f64("price")            // Option<f64>  (handles , → .)
self.cleaned_bool("active")          // Option<bool> (true/1/on → true)

// Undeclared field → guaranteed None, no leakage possible
self.cleaned_string("is_admin")      // None
```

> All `cleaned_*` variants return `Option`. For a default value: `.unwrap_or_default()` or `.unwrap_or(0)`.

---

## `Forms` helpers — numeric, date, UUID conversions

These methods are available directly on a `Forms` object (via `self.get_form()` or on a `form: Forms` variable). They require `is_valid()` to have been called first if the form was submitted.

### Direct conversions

```rust
form.get_i32("age")              // -> i32 (0 by default)
form.get_i64("count")            // -> i64 (0 by default)
form.get_u32("quantity")         // -> u32 (0 by default)
form.get_u64("id")               // -> u64 (0 by default)
form.get_f32("ratio")            // -> f32 (handles , → .)
form.get_f64("price")            // -> f64 (handles , → .)
form.get_bool("active")          // -> bool (true/1/on → true)
```

### Option conversions

```rust
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (handles , → .)
form.get_option_bool("news")     // -> Option<bool>
```

### Date / Time conversions

```rust
form.get_naive_date("birthday")           // -> NaiveDate (default if empty)
form.get_naive_time("meeting_time")       // -> NaiveTime (default if empty)
form.get_naive_datetime("event_start")    // -> NaiveDateTime (default if empty)
form.get_datetime_utc("created_at")       // -> DateTime<Utc> (Utc::now() if empty)

form.get_option_naive_date("birthday")        // -> Option<NaiveDate>
form.get_option_naive_time("meeting_time")    // -> Option<NaiveTime>
form.get_option_naive_datetime("event_start") // -> Option<NaiveDateTime>
form.get_option_datetime_utc("created_at")    // -> Option<DateTime<Utc>>
```

### UUID conversions

```rust
form.get_uuid("external_id")         // -> Uuid (Uuid::nil() if empty)
form.get_option_uuid("external_id")  // -> Option<Uuid>
```

> **💡** Float helpers (`get_f32`, `get_f64`, `get_option_f64`) automatically convert commas to dots (`19,99` → `19.99`) for French locales.

---

## Usage in `save()`

In `save()` methods, use `cleaned_*` to access data:

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            age: Set(self.cleaned_i32("age").unwrap_or(0)),
            website: Set(self.cleaned_string("website")),  // Option<String>
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

> **Password**: in `Auto` mode (default), `finalize()` hashes the password during `is_valid()`. In `save()`, the value is already hashed — reading with `cleaned_string("password")` is enough. Do not call `hash()` a second time.
> For a **login form** (verification, not storage), use `.no_hash()` when declaring the field.

---

## URL parameter access

### From `Request` — raw access

```rust
// Route parameter: /article/{id}
let id = request.path_param("id");       // Option<&str>

// Query string parameter: ?page=2
let page = request.from_url("page");     // Option<&str>
```

### From the form — `cleaned_*()` whitelisted and typed

The `cleaned_*` variants cover **all sources** (POST, path param, query param) in that priority order. They return `None` if the field is not declared in the form.

Practical example — pre-fill a field from the URL (`GET /edit?title=My+Article`):

```rust
pub async fn edit(
    mut request: Request,
    Prisme(mut form): Prisme<ArticleForm>,
) -> AppResult<Response> {
    if request.is_get() {
        if let Some(t) = form.cleaned_string("title") {
            form.get_form_mut().add_value("title", &t);
        }
    }

    if request.is_post() && form.is_valid().await {
        form.save(&request.engine.db).await?;
        return Ok(Redirect::to("/articles").into_response());
    }

    context_update!(request => { "form" => &form });
    request.render("edit.html")
}
```

> **Security** — `cleaned_*()` is bound to the form schema: an attacker cannot inject an arbitrary URL parameter (`?is_admin=true`) that is not a declared field. Works with both `#[form(...)]` and classic forms.

---

← [**RuniqueForm trait**](/docs/en/formulaire/trait) | [**Field types**](/docs/en/formulaire/fields) →
