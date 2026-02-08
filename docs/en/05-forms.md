
# Summary

- [📋 Forms](#overview)
  - [Overview](#overview)
  - [Prisme Extractor](#prisme-extractor)
  - [Manual Approach: RuniqueForm Trait](#manual-approach-runiqueform-trait)
    - [Basic Structure](#basic-structure)
    - [RuniqueForm Trait Methods](#runiqueform-trait-methods)
    - [Validation Pipeline is_valid()](#validation-pipeline-is_valid)
    - [Typed Conversion Helpers](#typed-conversion-helpers)
      - [Direct Conversions](#direct-conversions)
      - [Option Conversions (None if empty)](#option-conversions)
      - [Usage in save()](#usage-in-save)
  - [Field Types](#field-types)
    - [TextField — Text inputs](#textfield)
    - [NumericField — Numeric inputs](#numericfield)
    - [BooleanField — Checkboxes / Simple Radio](#booleanfield)
    - [ChoiceField — Select / Dropdown](#choicefield)
    - [RadioField — Radio Buttons](#radiofield)
    - [CheckboxField — Multiple Checkboxes](#checkboxfield)
    - [DateField, TimeField, DateTimeField — Date / Time](#date-time-duration-fields)
    - [DurationField — Duration](#durationfield)
    - [FileField — File Upload](#filefield)
    - [Associated JS Files](#associated-js)
    - [ColorField — Color Picker](#colorfield)
    - [SlugField — URL-friendly Slug](#slugfield)
    - [UUIDField](#uuidfield)
    - [JSONField — Textarea with JSON validation](#jsonfield)
    - [IPAddressField — IP Address](#ipaddressfield)
  - [Summary of Field Types](#field-types-recap)
  - [Automatic Approach: DeriveModelForm](#automatic-approach-deriveform)
    - [Auto-excluded Fields](#auto-excluded-fields)
    - [Automatic Type Detection](#automatic-type-detection)
    - [Customization Attributes](#customization-attributes)
  - [Database Errors](#database-errors)
  - [Rendering in Templates](#rendering-templates)
    - [Full Form](#full-form)
    - [Field by Field](#field-by-field)
    - [Global Errors](#global-errors)
    - [Field Data in JSON](#json-data)
  - [Full Example: Registration with Save](#full-example-registration)
    - [GET/POST Handler](#get-post-handler)
  - [⚠️ Common Pitfalls](#common-pitfalls)
    - [1. Template Variable Name Collision](#variable-collision)
    - [2. Forgetting mut on Form](#forgetting-mut)
    - [3. Comparing Passwords after is_valid()](#comparing-passwords)
  - [Next Steps](#next-steps)

---

<a id="overview"></a>

## Overview

Runique provides a powerful form system inspired by Django. There are **two approaches**:

1. **Manual** — Define fields via the `RuniqueForm` trait.
2. **Automatic** — Derive a form from a SeaORM model with `#[derive(DeriveModelForm)]`.

Forms are automatically extracted from requests via the **Prisme** extractor. They handle validation (including the `validator` crate for emails/URLs), CSRF, Argon2 password hashing, and can be saved directly to the database.

---

<a id="prisme-extractor"></a>

## Prisme Extractor

`Prisme<T>` is an Axum extractor that orchestrates a complete pipeline behind the scenes:

1. **Sentinel** — Checks access rules (login, roles) via `GuardRules`.
2. **Aegis** — Unique extraction of the body (multipart, urlencoded, json) normalized into a `HashMap`.
3. **CSRF Gate** — Verifies the CSRF token in the parsed data.
4. **Construction** — Creates form `T`, fills the fields, and triggers validation.

```rust
use runique::prelude::*;

pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            // Valid form → process logic
        }
    }
    // ...
}

```

> **💡** The developer simply writes `Prisme(mut form)` — the entire security pipeline is transparent.

---

<a id="manual-approach-runiqueform-trait"></a>

## Manual Approach: RuniqueForm Trait

<a id="basic-structure"></a>

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

> **💡 `impl_form_access!()**` automatically generates `from_form()`, `get_form()`, and `get_form_mut()`. If your field is not named `form`, pass the name as an argument: `impl_form_access!(my_form_field)`.

<details>
<summary>Equivalent without macro (for reference)</summary>

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

<a id="runiqueform-trait-methods"></a>

### RuniqueForm Trait Methods

| Method | Role |
| --- | --- |
| `register_fields(form)` | Declares form fields |
| `from_form(form)` | Builds the instance from a `Forms` object |
| `get_form()` / `get_form_mut()` | Accessors for the internal `Forms` |
| `clean()` | Cross-field business logic (e.g., `pwd1 == pwd2`) — **optional** |
| `is_valid()` | Full pipeline: field validation → `clean()` → `finalize()` |
| `database_error(&err)` | Injects a DB error into the correct field |
| `build(tera, csrf_token)` | Builds an empty form |
| `build_with_data(data, tera, csrf)` | Builds, populates, and validates |

<a id="validation-pipeline-is_valid"></a>

### Validation Pipeline `is_valid()`

Calling `form.is_valid().await` triggers **3 steps in order**:

1. **Field Validation** — Each field executes its `validate()`: required, length, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`...)
2. **`clean()`** — Custom business logic (passwords are still in plain text at this stage, allowing for `pwd1 == pwd2` comparisons).
3. **`finalize()`** — Final transformations (automatic Argon2 hashing of `Password` fields).

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

> **⚠️ Important**: After `is_valid()`, `Password` fields are **automatically hashed with Argon2**. Use `clean()` for any plain text password comparisons.

---

<a id="typed-conversion-helpers"></a>

## Typed Conversion Helpers

Form values are stored as `String`. Instead of parsing manually, use the typed helpers on `Forms`:

<a id="direct-conversions"></a>

### Direct Conversions

```rust
form.get_string("username")     // -> String ("" if empty)
form.get_i32("age")              // -> i32 (0 by default)
form.get_i64("count")            // -> i64 (0 by default)
form.get_u32("quantity")         // -> u32 (0 by default)
form.get_u64("id")               // -> u64 (0 by default)
form.get_f32("ratio")            // -> f32 (handles comma → dot)
form.get_f64("price")            // -> f64 (handles comma → dot)
form.get_bool("active")          // -> bool (true/1/on → true)

```

<a id="option-conversions"></a>

### Option Conversions (None if empty)

```rust
form.get_option("bio")           // -> Option<String> (None if empty)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("rating")    // -> Option<f64> (handles comma → dot)
form.get_option_bool("news")     // -> Option<bool>

```

<a id="usage-in-save"></a>

### Usage in save()

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

> **💡** Float helpers (`get_f32`, `get_f64`, `get_option_f64`) automatically convert commas to dots (`19,99` → `19.99`) for French/European locales.

---

<a id="field-types"></a>

## Field Types

<a id="textfield"></a>

### TextField — Text inputs

`TextField` handles 6 special formats via the `SpecialFormat` enum:

```rust
// Simple text
form.field(&TextField::text("username").label("Name").required());

// Email — validated via `validator::ValidateEmail`
form.field(&TextField::email("email").label("Email").required());

// URL — validated via `validator::ValidateUrl`
form.field(&TextField::url("website").label("Website"));

// Password — automatic Argon2 hashing in finalize(), never re-displayed in HTML
form.field(
    &TextField::password("password")
        .label("Password")
        .required()
        .min_length(8, "Min 8 characters"),
);

// Textarea
form.field(&TextField::textarea("summary").label("Summary"));

// RichText — automatic XSS sanitization before validation
form.field(&TextField::richtext("content").label("Content"));

```

**Builder Options:**

```rust
TextField::text("name")
    .label("My field")                // Displayed label
    .placeholder("Enter...")          // Placeholder
    .required()                       // Mandatory (default message)
    .min_length(3, "Too short")       // Min length with message
    .max_length(100, "Too long")      // Max length with message
    .readonly("Read only")            // Read-only
    .disabled("Disabled")             // Disabled

```

**Automatic Behaviors per Format:**

| Format | Validation | Transformation |
| --- | --- | --- |
| `Email` | `validator::ValidateEmail` | Conversion to lowercase |
| `Url` | `validator::ValidateUrl` | — |
| `Password` | Standard | Argon2 hashing in `finalize()`, value cleared on `render()` |
| `RichText` | Standard | XSS Sanitization (`sanitize()`) before validation |
| `Csrf` | Session Token | — |

**Password Utilities:**

```rust
// Hash manually
let hash = field.hash_password()?;

// Verify a password
let ok = TextField::verify_password("plain_pwd", "$argon2...");

```

> Automatic hashing detects if the value already starts with `$argon2` to avoid double hashing.

<a id="numericfield"></a>

### NumericField — Numeric inputs

5 variants via the `NumericConfig` enum:

```rust
// Integer with bounds
form.field(
    &NumericField::integer("age")
        .label("Age")
        .min(0.0, "Min 0")
        .max(150.0, "Max 150"),
);

// Floating point number
form.field(&NumericField::float("price").label("Price"));

// Decimal number with precision
form.field(
    &NumericField::decimal("amount")
        .label("Amount")
        .digits(2, 4),  // min 2, max 4 digits after the decimal point
);

// Percentage (0–100 by default)
form.field(&NumericField::percent("rate").label("Rate"));

// Range slider with min, max, default value
form.field(
    &NumericField::range("volume", 0.0, 100.0, 50.0)
        .label("Volume")
        .step(5.0),
);

```

**Options:** `.min(val, msg)`, `.max(val, msg)`, `.step(val)`, `.digits(min, max)`, `.label(l)`, `.placeholder(p)`

<a id="booleanfield"></a>

### BooleanField — Checkboxes / Simple Radio

```rust
// Simple checkbox
form.field(
    &BooleanField::new("accept_terms")
        .label("I accept the terms")
        .required(),
);

// Simple radio (yes/no)
form.field(&BooleanField::radio("newsletter").label("Newsletter"));

// Pre-checked
form.field(&BooleanField::new("remember_me").label("Remember me").checked());

```

<a id="choicefield"></a>

### ChoiceField — Select / Dropdown

```rust
use runique::forms::fields::choice::ChoiceOption;

let choices = vec![
    ChoiceOption::new("fr", "France"),
    ChoiceOption::new("uk", "United Kingdom"),
    ChoiceOption::new("us", "USA"),
];

// Simple Select
form.field(
    &ChoiceField::new("country")
        .label("Country")
        .choices(choices.clone())
        .required(),
);

// Multiple Select
form.field(
    &ChoiceField::new("languages")
        .label("Languages")
        .choices(choices)
        .multiple(),
);

```

> Validation automatically verifies that the submitted value is part of the declared choices.

<a id="radiofield"></a>

### RadioField — Radio Buttons

```rust
form.field(
    &RadioField::new("gender")
        .label("Gender")
        .choices(vec![
            ChoiceOption::new("m", "Male"),
            ChoiceOption::new("f", "Female"),
            ChoiceOption::new("o", "Other"),
        ])
        .required(),
);

```

<a id="checkboxfield"></a>

### CheckboxField — Multiple Checkboxes

```rust
form.field(
    &CheckboxField::new("hobbies")
        .label("Hobbies")
        .choices(vec![
            ChoiceOption::new("sport", "Sport"),
            ChoiceOption::new("music", "Music"),
            ChoiceOption::new("reading", "Reading"),
        ]),
);

```

> Submitted values are in the format `"val1,val2,val3"`. Validation checks that each value exists in the choices.

<a id="date-time-duration-fields"></a>

### DateField, TimeField, DateTimeField — Date / Time

```rust
use chrono::NaiveDate;

// Date (format: YYYY-MM-DD)
form.field(
    &DateField::new("birthday")
        .label("Date of Birth")
        .min(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), "Too old")
        .max(NaiveDate::from_ymd_opt(2010, 12, 31).unwrap(), "Too young"),
);

// Time (format: HH:MM)
form.field(&TimeField::new("meeting_time").label("Meeting Time"));

// Date + Time (format: YYYY-MM-DDTHH:MM)
form.field(&DateTimeField::new("event_start").label("Event Start"));

```

<a id="durationfield"></a>

### DurationField — Duration

```rust
form.field(
    &DurationField::new("timeout")
        .label("Timeout (seconds)")
        .min_seconds(60, "Minimum 1 minute")
        .max_seconds(3600, "Maximum 1 hour"),
);

```

<a id="filefield"></a>

### FileField — File Upload

```rust
use runique::config::StaticConfig;

let config = StaticConfig::from_env();

// Image with full constraints
form.field(
    &FileField::image("avatar")
        .label("Profile Picture")
        .upload_to(&config)
        .max_size_mb(5)
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp"]),
);

// Document
form.field(
    &FileField::document("cv")
        .label("CV")
        .max_size_mb(10),
);

// Any file (multi-file)
form.field(
    &FileField::any("attachments")
        .label("Attachments")
        .max_files(5),
);

```

> **Security**: `.svg` files are **always rejected** by default (XSS risk). Image validation uses the `image` crate to verify the actual file format.

<a id="associated-js"></a>

### Associated JS Files

```rust
fn register_fields(form: &mut Forms) {
    // ... fields ...
    form.add_js(&["js/my_script.js", "js/other.js"]);
}

```

JS files are automatically included in the HTML rendering of the form.

<a id="colorfield"></a>

### ColorField — Color Picker

```rust
form.field(
    &ColorField::new("theme_color")
        .label("Theme Color")
        .default_color("#3498db"),  // Validates #RGB or #RRGGBB format
);

```

<a id="slugfield"></a>

### SlugField — URL-friendly Slug

```rust
form.field(
    &SlugField::new("slug")
        .label("Slug")
        .placeholder("my-article-url")
        .allow_unicode(),  // Optional: allows unicode characters
);

```

> Validation: letters, numbers, hyphens, underscores only. Cannot start or end with a hyphen.

<a id="uuidfield"></a>

### UUIDField

```rust
form.field(
    &UUIDField::new("external_id")
        .label("External ID")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
);

```

<a id="jsonfield"></a>

### JSONField — Textarea with JSON validation

```rust
form.field(
    &JSONField::new("metadata")
        .label("Metadata")
        .placeholder(r#"{"key": "value"}"#)
        .rows(10),  // Number of rows in the textarea
);

```

<a id="ipaddressfield"></a>

### IPAddressField — IP Address

```rust
// IPv4 + IPv6
form.field(&IPAddressField::new("server_ip").label("Server IP"));

// IPv4 only
form.field(&IPAddressField::new("gateway").label("Gateway").ipv4_only());

// IPv6 only
form.field(&IPAddressField::new("ipv6").label("IPv6 Address").ipv6_only());

```

---

<a id="field-types-recap"></a>

## Summary of Field Types

| Struct | Constructors | Special Validation |
| --- | --- | --- |
| `TextField` | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL via `validator`, Argon2, XSS sanitization |
| `NumericField` | `integer()`, `float()`, `decimal()`, `percent()`, `range()` | Min/max bounds, decimal precision |
| `BooleanField` | `new()`, `radio()` | Required = must be checked |
| `ChoiceField` | `new()` + `.multiple()` | Value must be in declared choices |
| `RadioField` | `new()` | Value must be in declared choices |
| `CheckboxField` | `new()` | All values must be in choices |
| `DateField` | `new()` | Format `YYYY-MM-DD`, min/max bounds |
| `TimeField` | `new()` | Format `HH:MM`, min/max bounds |
| `DateTimeField` | `new()` | Format `YYYY-MM-DDTHH:MM`, min/max bounds |
| `DurationField` | `new()` | Seconds, min/max bounds |
| `FileField` | `image()`, `document()`, `any()` | Extensions, size, dimensions, anti-SVG |
| `ColorField` | `new()` | `#RRGGBB` or `#RGB` format |
| `SlugField` | `new()` | ASCII/unicode, no hyphen at start/end |
| `UUIDField` | `new()` | Valid UUID format |
| `JSONField` | `new()` | Valid JSON via `serde_json` |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()` | IPv4/IPv6 via `std::net::IpAddr` |

---

<a id="automatic-approach-deriveform"></a>

## Automatic Approach: DeriveModelForm

For simple cases, derive a form directly from a SeaORM model:

```rust
use runique::prelude::*;

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

// Automatically generates: pub struct ModelForm { pub form: Forms }
#[derive(DeriveModelForm)]
pub struct Model;

```

<a id="auto-excluded-fields"></a>

### Auto-excluded Fields

`DeriveModelForm` automatically excludes:

* `id` (primary key)
* `csrf_token`
* `created_at`, `updated_at`
* `is_active`, `deleted_at`
* Any field marked with `#[sea_orm(primary_key)]`

<a id="automatic-type-detection"></a>

### Automatic Type Detection

| Rule | Generated Field Type | `to_active_model()` Helper |
| --- | --- | --- |
| Name contains `email` | `TextField::email()` | `get_string()` |
| Name contains `password` / `pwd` | `TextField::password()` | `get_string()` |
| Name contains `url` / `link` / `website` | `TextField::url()` | `get_string()` |
| `String` + name `description` / `bio` / `content` / `message` | `TextField::textarea()` | `get_string()` |
| `String` (other) | `TextField::text()` | `get_string()` |
| `i32` | `NumericField::integer()` | `get_i32()` |
| `i64` | `NumericField::integer()` | `get_i64()` |
| `u32` | `NumericField::integer()` | `get_u32()` |
| `u64` | `NumericField::integer()` | `get_u64()` |
| `f32` | `NumericField::float()` | `get_f32()` |
| `f64` | `NumericField::float()` | `get_f64()` |
| `bool` | `BooleanField::new()` | `get_bool()` |
| `Option<T>` | **Not required** field | `get_option()` |
| Non-`Option<T>` | **Required** field | Matching type |

<a id="customization-attributes"></a>

### Customization Attributes

```rust
#[derive(DeriveModelForm)]
#[exclude(bio, age)]  // Exclude additional fields
pub struct Model;

```

---

<a id="database-errors"></a>

## Database Errors

The `database_error()` method automatically parses DB errors to report them on the correct field:

```rust
match form.save(&request.engine.db).await {
    Ok(_) => { /* success */ }
    Err(err) => {
        form.database_error(&err);
        // Error is set on the relevant field
    }
}

```

**Supported error formats:**

* **PostgreSQL**: `UNIQUE constraint`, `Key (field)=(value)`
* **SQLite**: `UNIQUE constraint failed: table.field`
* **MySQL**: `Duplicate entry ... for key 'table.field'`

If the field is identified, the error appears on that field (e.g., "This email is already taken"). Otherwise, it is added to the global errors.

---

<a id="rendering-templates"></a>

## Rendering in Templates

<a id="full-form"></a>

### Full Form

```html
<form method="post">
    {% form.register_form %}
    <button type="submit">Register</button>
</form>

```

Automatically renders: all fields, labels, validation errors, the CSRF token, and JS scripts.

<a id="field-by-field"></a>

### Field by Field

```html
<form method="post">
    {% csrf %} <div class="row">
        <div class="col-6">{% form.register_form.username %}</div>
        <div class="col-6">{% form.register_form.email %}</div>
    </div>
    {% form.register_form.password %}
    <button type="submit">Register</button>
</form>

```

<a id="global-errors"></a>

### Global Errors

```html
{% if register_form.global_errors %}
    <div class="alert alert-danger">
        {% for msg in register_form.global_errors %}
            <p>{{ msg }}</p>
        {% endfor %}
    </div>
{% endif %}

```

<a id="json-data"></a>

### Field Data in JSON

Forms automatically serialize `data`, `errors`, `global_errors`, `html`, `rendered_fields`, `fields`, and `js_files`.

---

<a id="full-example-registration"></a>

## Full Example: Registration with Save

```rust
use runique::prelude::*;

#[derive(Serialize, Debug, Clone)]
#[serde(transparent)]
pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(
            &TextField::text("username")
                .label("Username")
                .required(),
        );

        form.field(
            &TextField::email("email")
                .label("Email")
                .required(),
        );

        form.field(
            &TextField::password("password")
                .label("Password")
                .required()
                .min_length(8, "Minimum 8 characters"),
        );
    }

    impl_form_access!();
}

impl RegisterForm {
    pub async fn save(&self, db: &DatabaseConnection) -> Result<users::Model, DbErr> {
        use sea_orm::Set;
        let model = users::ActiveModel {
            username: Set(self.form.get_string("username")),
            email: Set(self.form.get_string("email")),
            // Password is already Argon2 hashed after is_valid()
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };
        model.insert(db).await
    }
}

```

<a id="get-post-handler"></a>

### GET/POST Handler

```rust
pub async fn register(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "profile/register_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Register",
            "register_form" => &form,
        });
        return request.render(template);
    }

    if request.is_post() {
        if form.is_valid().await {
            match form.save(&request.engine.db).await {
                Ok(_) => {
                    success!(request.notices => "Registration successful!");
                    return Ok(Redirect::to("/").into_response());
                }
                Err(err) => {
                    form.database_error(&err);
                }
            }
        }

        context_update!(request => {
            "title" => "Error",
            "register_form" => &form,
            "messages" => flash_now!(error => "Please correct the errors"),
        });
        return request.render(template);
    }

    request.render(template)
}

```

---

<a id="common-pitfalls"></a>

## ⚠️ Common Pitfalls

<a id="variable-collision"></a>

### 1. Template Variable Name Collision

If your template uses `{% form.user %}`, the `user` variable in the context **must** be a form, not a SeaORM Model:

```rust
// ❌ ERROR — db_user is a Model, not a form
context_update!(request => { "user" => &db_user });

// ✅ CORRECT — separate names
context_update!(request => {
    "user_form" => &form,
    "found_user" => &db_user,
});

```

<a id="forgetting-mut"></a>

### 2. Forgetting `mut` on Form

```rust
// ❌ Cannot call is_valid()
Prisme(form): Prisme<MyForm>

// ✅ Correct
Prisme(mut form): Prisme<MyForm>

```

<a id="comparing-passwords"></a>

### 3. Comparing Passwords after `is_valid()`

```rust
// ❌ After is_valid(), passwords are hashed!
let pwd = form.get_form().get_string("password");
// pwd == "$argon2id$v=19$m=..." 😱

// ✅ Compare in clean(), BEFORE finalization
async fn clean(&mut self) -> Result<(), StrMap> {
    let pwd1 = self.form.get_string("password");
    let pwd2 = self.form.get_string("password_confirm");
    if pwd1 != pwd2 { /* error */ }
    Ok(())
}

```

---

<a id="next-steps"></a>

## Next Steps

← **[Routing](https://github.com/seb-alliot/runique/blob/refonte-builder-app/docs/en/04-routing.md)** | **[Templates](https://github.com/seb-alliot/runique/blob/refonte-builder-app/docs/en/06-templates.md)** →

---

