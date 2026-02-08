
# 📋 Forms

## Table of Contents

1. [Overview](#overview)
2. [Prisme Extractor](#prisme-extractor)
3. [Manual Approach: `RuniqueForm` Trait](#manual-approach-runiqueform-trait)
    1. [Basic Structure](#basic-structure)
    2. [Methods of the `RuniqueForm` Trait](#methods-of-the-runiqueform-trait)
    3. [Validation Pipeline `is_valid()`](#validation-pipeline-is_valid)
    4. [Typed Conversion Helpers](#typed-conversion-helpers)
        1. [Direct Conversions](#direct-conversions)
        2. [Option Conversions](#option-conversions)
        3. [Usage in `save()`](#usage-in-save)
4. [Field Types](#field-types)
    1. [TextField — Text Fields](#textfield---text-fields)
    2. [NumericField — Numeric Fields](#numericfield---numeric-fields)
    3. [BooleanField — Checkboxes / Simple Radio](#booleanfield---checkboxes--simple-radio)
    4. [ChoiceField — Select / Dropdown](#choicefield---select--dropdown)
    5. [RadioField — Radio Buttons](#radiofield---radio-buttons)
    6. [CheckboxField — Multiple Checkboxes](#checkboxfield---multiple-checkboxes)
    7. [DateField, TimeField, DateTimeField — Date / Time](#datefield-timefield-datetimefield---date--time)
    8. [DurationField — Duration](#durationfield---duration)
    9. [FileField — File Uploads](#filefield---file-uploads)
    10. [Associated JS Files](#associated-js-files)
    11. [ColorField — Color Picker](#colorfield---color-picker)
    12. [SlugField — URL-friendly Slug](#slugfield---url-friendly-slug)
    13. [UUIDField](#uuidfield)
    14. [JSONField — Textarea with JSON Validation](#jsonfield---textarea-with-json-validation)
    15. [IPAddressField — IP Address](#ipaddressfield---ip-address)
5. [Summary of Field Types](#summary-of-field-types)
6. [Automatic Approach: `DeriveModelForm`](#automatic-approach-derivemodelform)
    1. [Auto-excluded Fields](#auto-excluded-fields)
    2. [Automatic Type Detection](#automatic-type-detection)
    3. [Customization Attributes](#customization-attributes)
7. [Database Errors](#database-errors)
8. [Template Rendering](#template-rendering)
    1. [Full Form](#full-form)
    2. [Field by Field](#field-by-field)
    3. [Global Errors](#global-errors)
    4. [Field Data in JSON](#field-data-in-json)
9. [Complete Example: Registration with Save](#complete-example-registration-with-save)
    1. [GET/POST Handler](#getpost-handler)
10. [⚠️ Common Pitfalls](#common-pitfalls)
    1. [Template Variable Name Collision](#template-variable-name-collision)
    2. [Forgetting `mut` on Form](#forgetting-mut-on-form)
    3. [Comparing Passwords after `is_valid()`](#comparing-passwords-after-is_valid)
11. [Next Steps](#next-steps)

---

## Overview

Runique provides a powerful form system inspired by Django. There are **two approaches**:

1. **Manual** — Define fields via the `RuniqueForm` trait
2. **Automatic** — Derive a form from a SeaORM model using `#[derive(DeriveModelForm)]`

Forms are automatically extracted from requests via the **Prisme** extractor, handle validation (including email/URL validation via the `validator` crate), CSRF protection, Argon2 password hashing, and can be saved directly to the database.

---

## Prisme Extractor

`Prisme<T>` is an Axum extractor that orchestrates a full pipeline behind the scenes:

1. **Sentinel** — Checks access rules (login, roles) via `GuardRules`
2. **Aegis** — Single extraction of the body (multipart, urlencoded, json) normalized into a `HashMap`
3. **CSRF Gate** — Validates the CSRF token in the parsed data
4. **Construction** — Creates the `T` form, fills fields, triggers validation

```rust
use runique::prelude::*;

pub async fn registration(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            // Valid form → process
        }
    }
    // ...
}
```

> **💡** The developer just writes `Prisme(mut form)` — the entire security pipeline is transparent.

---

## Manual Approach: `RuniqueForm` Trait

### Basic Structure

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

> **💡 `impl_form_access!()`** automatically generates `from_form()`, `get_form()`, and `get_form_mut()`. If your field is not named `form`, pass the field name as an argument: `impl_form_access!(formulaire);`

### Methods of the `RuniqueForm` Trait

| Method                              | Purpose                                                                    |
| ----------------------------------- | -------------------------------------------------------------------------- |
| `register_fields(form)`             | Declares the form fields                                                   |
| `from_form(form)`                   | Constructs the instance from a `Forms`                                     |
| `get_form()` / `get_form_mut()`     | Accessors to the internal `Forms`                                          |
| `clean()`                           | Cross-field business logic (e.g., `password1 == password2`) — **optional** |
| `is_valid()`                        | Full pipeline: field validation → `clean()` → `finalize()`                 |
| `database_error(&err)`              | Injects a DB error on the correct field                                    |
| `build(tera, csrf_token)`           | Builds an empty form                                                       |
| `build_with_data(data, tera, csrf)` | Builds, fills, and validates                                               |

### Validation Pipeline `is_valid()`

Calling `form.is_valid().await` triggers **3 steps in order**:

1. **Field validation** — Each field runs its `validate()`: required, length, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`, etc.)
2. **`clean()`** — Custom business logic (passwords are still in plaintext at this stage, allowing comparison `password1 == password2`)
3. **`finalize()`** — Final transformations (automatic Argon2 hashing of `Password` fields)

```rust
#[async_trait::async_trait]
impl RuniqueForm for RegisterForm {
    async fn clean(&mut self) -> Result<(), StrMap> {
        let password1 = self.form.get_string("password");
        let password2 = self.form.get_string("password_confirm");

        if password1 != password2 {
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

> **⚠️ Important**: After `is_valid()`, `Password` fields are **automatically hashed using Argon2**. Use `clean()` for any plaintext password comparison.

---

## Typed Conversion Helpers

### Direct Conversions

```rust
form.get_string("username")     // -> String ("" if empty)
form.get_i32("age")             // -> i32 (0 by default)
form.get_i64("count")           // -> i64 (0 by default)
form.get_u32("quantity")        // -> u32 (0 by default)
form.get_u64("id")              // -> u64 (0 by default)
form.get_f32("ratio")           // -> f32 (handles , → .)
form.get_f64("price")           // -> f64 (handles , → .)
form.get_bool("active")         // -> bool (true/1/on → true)
```

### Option Conversions

```rust
form.get_option("bio")          // -> Option<String> (None if empty)
form.get_option_i32("age")      // -> Option<i32>
form.get_option_i64("score")    // -> Option<i64>
form.get_option_f64("note")     // -> Option<f64> (handles , → .)
form.get_option_bool("news")    // -> Option<bool>
```

### Usage in `save()`

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

> **💡** Float helpers automatically convert commas to dots for French locales.

---

## Field Types

### TextField — Text Fields

```rust
form.field(&TextField::text("username").label("Name").required());
form.field(&TextField::email("email").label("Email").required());
form.field(&TextField::url("website").label("Website"));
form.field(&TextField::password("password").label("Password").required().min_length(8, "Min 8 characters"));
form.field(&TextField::textarea("summary").label("Summary"));
form.field(&TextField::richtext("content").label("Content"));
```

**Password utilities:**

```rust
let hash = field.hash_password()?;
let ok = TextField::verify_password("plain_pw", "$argon2...");
```

---

### NumericField — Numeric Fields

```rust
form.field(&NumericField::integer("age").label("Age").min(0.0, "Min 0").max(150.0, "Max 150"));
form.field(&NumericField::float("price").label("Price"));
form.field(&NumericField::decimal("amount").label("Amount").digits(2,4));
form.field(&NumericField::percent("rate").label("Rate"));
form.field(&NumericField::range("volume",0.0,100.0,50.0).label("Volume").step(5.0));
```

---

### BooleanField — Checkboxes / Simple Radio

```rust
form.field(&BooleanField::new("accept_terms").label("I accept the terms").required());
form.field(&BooleanField::radio("newsletter").label("Newsletter"));
form.field(&BooleanField::new("remember_me").label("Remember me").checked());
```

---

### ChoiceField — Select / Dropdown

```rust
let choices = vec![
    ChoiceOption::new("fr", "France"),
    ChoiceOption::new("be", "Belgium"),
    ChoiceOption::new("ch", "Switzerland"),
];

form.field(&ChoiceField::new("country").label("Country").choices(choices.clone()).required());
form.field(&ChoiceField::new("languages").label("Languages").choices(choices).multiple());
```

---

### RadioField — Radio Buttons

```rust
form.field(&RadioField::new("gender").label("Gender").choices(vec![
    ChoiceOption::new("m", "Male"),
    ChoiceOption::new("f", "Female"),
    ChoiceOption::new("o", "Other"),
]).required());
```

---

### CheckboxField — Multiple Checkboxes

```rust
form.field(&CheckboxField::new("hobbies").label("Hobbies").choices(vec![
    ChoiceOption::new("sport","Sport"),
    ChoiceOption::new("music","Music"),
    ChoiceOption::new("reading","Reading"),
]));
```

---

### DateField, TimeField, DateTimeField — Date / Time

```rust
form.field(&DateField::new("birthday").label("Birth date").min(NaiveDate::from_ymd_opt(1900,1,1).unwrap(),"Too old").max(NaiveDate::from_ymd_opt(2010,12,31).unwrap(),"Too recent"));
form.field(&TimeField::new("meeting_time").label("Meeting time"));
form.field(&DateTimeField::new("event_start").label("Event start"));
```

---

### DurationField — Duration

```rust
form.field(&DurationField::new("timeout").label("Timeout (seconds)").min_seconds(60,"Minimum 1 minute").max_seconds(3600,"Maximum 1 hour"));
```

---

### FileField — File Uploads

```rust
form.field(&FileField::image("avatar").label("Profile picture").upload_to(&config).max_size_mb(5).max_files(1).max_dimensions(1920,1080).allowed_extensions(vec!["png","jpg","jpeg","webp"]));
form.field(&FileField::document("cv").label("CV").max_size_mb(10));
form.field(&FileField::any("attachments").label("Attachments").max_files(5));
```

---

### Associated JS Files

```rust
form.add_js(&["js/my_script.js","js/other.js"]);
```

---

### ColorField — Color Picker

```rust
form.field(&ColorField::new("theme_color").label("Theme color").default_color("#3498db"));
```

---

### SlugField — URL-friendly Slug

```rust
form.field(&SlugField::new("slug").label("Slug").placeholder("my-article-url").allow_unicode());
```

---

### UUIDField

```rust
form.field(&UUIDField::new("external_id").label("External ID").placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"));
```

---

### JSONField — Textarea with JSON Validation

```rust
form.field(&JSONField::new("metadata").label("Metadata").placeholder(r#"{"key":"value"}"#).rows(10));
```

---

### IPAddressField — IP Address

```rust
form.field(&IPAddressField::new("server_ip").label("Server IP"));
form.field(&IPAddressField::new("gateway").label("Gateway").ipv4_only());
form.field(&IPAddressField::new("ipv6").label("IPv6 Address").ipv6_only());
```

---

## Summary of Field Types

| Struct | Constructors | Special Validation |
|--------|-------------|------------------|
| `TextField` | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL, Argon2, XSS sanitization |
| `NumericField` | `integer()`, `float()`, `decimal()`, `percent()`, `range()` | Min/max, decimal precision |
| `BooleanField` | `new()`, `radio()` | Required |
| `ChoiceField` | `new()` + `.multiple()` | Value must be among choices |
| `RadioField` | `new()` | Value must be among choices |
| `CheckboxField` | `new()` | All values must be among choices |
| `DateField` | `new()` | YYYY-MM-DD, min/max |
| `TimeField` | `new()` | HH:MM, min/max |
| `DateTimeField` | `new()` | YYYY-MM-DDTHH:MM, min/max |
| `DurationField` | `new()` | Seconds, min/max |
| `FileField` | `image()`, `document()`, `any()` | Extensions, size, anti-SVG |
| `ColorField` | `new()` | #RGB / #RRGGBB |
| `SlugField` | `new()` | ASCII/unicode, no dash at start/end |
| `UUIDField` | `new()` | UUID format |
| `JSONField` | `new()` | Valid JSON |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()` | IPv4/IPv6 |

---

## Automatic Approach: `DeriveModelForm`

```rust
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub age: Option<i32>,
    pub is_active: bool,
    pub created_at: DateTime,
}

#[derive(DeriveModelForm)]
pub struct Model;
```

### Auto-excluded Fields

- `id`, `csrf_token`, `created_at`, `updated_at`, `is_active`, `deleted_at`

### Automatic Type Detection

| Rule | Field Type | Helper |
|------|-----------|--------|
| Name contains `email` | `TextField::email()` | `get_string()` |
| Name contains `password` | `TextField::password()` | `get_string()` |
| Name contains `url` / `link` | `TextField::url()` | `get_string()` |
| `String` + `description` / `bio` | `TextField::textarea()` | `get_string()` |
| `i32` | `NumericField::integer()` | `get_i32()` |
| `f32` | `NumericField::float()` | `get_f32()` |
| `bool` | `BooleanField::new()` | `get_bool()` |
| `Option<T>` | Non-required | `get_option()` |

### Customization Attributes

```rust
#[derive(DeriveModelForm)]
#[exclude(bio, age)]
pub struct Model;
```

---

## Database Errors

```rust
match form.save(&request.engine.db).await {
    Ok(_) => {},
    Err(err) => { form.database_error(&err); }
}
```

---

## Template Rendering

### Full Form

```html
<form method="post">{% form.registration_form %}<button type="submit">Register</button></form>
````


````

### Field by Field

```html
<label>Username</label>{{ form.username }}
<label>Email</label>{{ form.email }}
````

### Global Errors

```html
{% for error in form.errors.global %}
<p class="error">{{ error }}</p>
{% endfor %}
```

### Field Data in JSON

```html
<script>
let data = {{ form.to_json() | safe }};
</script>
```

---

## Complete Example: Registration with Save

### GET/POST Handler

```rust
pub async fn handler(Prisme(mut form): Prisme<RegisterForm>, db: DatabaseConnection) -> AppResult<Response> {
    if form.is_valid().await {
        form.save(&db).await?;
    }
    Ok(render_template("register.html", &form))
}
```

---

## ⚠️ Common Pitfalls

### Template Variable Name Collision

Avoid using `username` in the template if you have a variable with the same name.

### Forgetting `mut` on Form

`Prisme(form)` → cannot mutate → call `Prisme(mut form)` instead.

### Comparing Passwords after `is_valid()`

`Password` fields are already hashed after `is_valid()`. Compare in `clean()` instead.

---

## Next Steps

* Extend field types for custom UI (tags, color pickers, sliders)
* Add dynamic validation rules via `runique::validators`
* Integrate `Prisme` into other Axum routes
* Improve JS helper files for live validation

```

