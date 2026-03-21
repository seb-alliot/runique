# Typed conversion helpers

[← RuniqueForm trait](/docs/en/formulaire/trait)

---

Form values are stored as `String`. Instead of parsing manually, use the typed helpers on `Forms`:

## Direct conversions

```rust
form.get_string("username")     // -> String ("" if empty)
form.get_i32("age")              // -> i32 (0 by default)
form.get_i64("count")            // -> i64 (0 by default)
form.get_u32("quantity")         // -> u32 (0 by default)
form.get_u64("id")               // -> u64 (0 by default)
form.get_f32("ratio")            // -> f32 (handles , → .)
form.get_f64("price")            // -> f64 (handles , → .)
form.get_bool("active")          // -> bool (true/1/on → true)
```

## Option conversions

```rust
form.get_option("bio")           // -> Option<String> (None if empty)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (handles , → .)
form.get_option_bool("news")     // -> Option<bool>
```

## Date / Time conversions

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

## UUID conversions

```rust
form.get_uuid("external_id")         // -> Uuid (Uuid::nil() if empty)
form.get_option_uuid("external_id")  // -> Option<Uuid>
```

## Usage in save()

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            password: Set(self.form.get_string("password")),
            age: Set(self.form.get_i32("age")),
            website: Set(self.form.get_option("website")),  // Option<String>
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

> **💡** Float helpers (`get_f32`, `get_f64`, `get_option_f64`) automatically convert commas to dots (`19,99` → `19.99`) for French locales.

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

```rust
form.cleaned_string("search")   // Option<String>
form.cleaned_i32("page")        // Option<i32>
form.cleaned_i64("id")          // Option<i64>
form.cleaned_u32("quantity")    // Option<u32>
form.cleaned_u64("ref")         // Option<u64>
form.cleaned_f32("ratio")       // Option<f32>  (handles , → .)
form.cleaned_f64("price")       // Option<f64>  (handles , → .)
form.cleaned_bool("active")     // Option<bool> (true/1/on → true)

// Unknown field → guaranteed None, no leakage possible
form.cleaned_string("is_admin") // None
```

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
