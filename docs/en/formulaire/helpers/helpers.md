# Typed conversion helpers

[← RuniqueForm trait](/docs/en/formulaire/trait)

---

Form values are stored as `String`. Use the `cleaned_*` typed helpers to convert them securely (whitelisted) and with proper typing.

## Recommended API — `cleaned_*()`

Methods on the `RuniqueForm` trait. They are whitelisted (undeclared field → `None`) and cover sources by priority: POST > Route parameter > Query string.

### Basics and Numerics

```rust
self.cleaned_string("username")      // Option<String> (None if empty)
self.cleaned_i32("age")              // Option<i32>
self.cleaned_i64("count")            // Option<i64>
self.cleaned_u32("quantity")         // Option<u32>
self.cleaned_u64("id")               // Option<u64>
self.cleaned_f32("ratio")            // Option<f32> (handles , → .)
self.cleaned_f64("price")            // Option<f64> (handles , → .)
self.cleaned_bool("active")          // Option<bool> (true/1/on → true)
```

### Date / Time / UUID

```rust
self.cleaned_naive_date("birthday")           // Option<NaiveDate> (YYYY-MM-DD format)
self.cleaned_naive_time("meeting_time")       // Option<NaiveTime> (HH:MM format)
self.cleaned_naive_datetime("event_start")    // Option<NaiveDateTime> (YYYY-MM-DDTHH:MM)
self.cleaned_datetime_utc("created_at")       // Option<DateTime<Utc>> (RFC3339)

self.cleaned_uuid("external_id")              // Option<Uuid>
```

> **💡** Float helpers (`cleaned_f32`, `cleaned_f64`) automatically convert commas to dots (`19,99` → `19.99`) to simplify input.
> 
> **Security Note:** All `cleaned_*` variants return `Option`. For a default value: `.unwrap_or_default()` or `.unwrap_or(0)`.

---

## Usage in `save()`

In your `save()` methods, exclusively use `cleaned_*` to extract data:

```rust
impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        let model = users::ActiveModel {
            username: Set(self.cleaned_string("username").unwrap_or_default()),
            email: Set(self.cleaned_string("email").unwrap_or_default()),
            age: Set(self.cleaned_i32("age").unwrap_or(0)),
            website: Set(self.cleaned_string("website")), // Option<String>
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

---

## URL parameter access

### From `Request` (Raw access)

```rust
// Route parameter: /article/{id}
let id = request.path_param("id");       // Option<&str>

// Query string parameter: ?page=2
let page = request.from_url("page");     // Option<&str>
```

### From the form (Recommended)

The `cleaned_*` variants automatically merge data from the request body and the URL.

Example — pre-fill a field from the URL (`GET /edit?title=My+Article`):

```rust
if request.is_get() {
    if let Some(t) = form.cleaned_string("title") {
        form.get_form_mut().add_value("title", &t);
    }
}
```

---

[← RuniqueForm trait](/docs/en/formulaire/trait) | [**Field types**](/docs/en/formulaire/fields) →
