# Table of Contents

- [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Prisme extractor](#prisme-extractor)
  - [Manual approach: RuniqueForm trait](#manual-approach-runiqueform-trait)
    - [Base structure](#base-structure)
    - [RuniqueForm trait methods](#runiqueform-trait-methods)
    - [`is_valid()` validation pipeline](#is_valid-validation-pipeline)
  - [Typed conversion helpers](#typed-conversion-helpers)
    - [Direct conversions](#direct-conversions)
    - [Option conversions](#option-conversions)
    - [Usage in save()](#usage-in-save)
  - [Field types](#field-types)
    - [TextField — Text fields](#textfield--text-fields)
    - [NumericField — Numeric fields](#numericfield--numeric-fields)
    - [BooleanField — Checkboxes / Single radio](#booleanfield--checkboxes--single-radio)
    - [ChoiceField — Select / Dropdown](#choicefield--select--dropdown)
    - [RadioField — Radio buttons](#radiofield--radio-buttons)
    - [CheckboxField — Multiple checkboxes](#checkboxfield--multiple-checkboxes)
    - [DateField, TimeField, DateTimeField — Date / Time](#datefield-timefield-datetimefield--date--time)
    - [DurationField — Duration](#durationfield--duration)
    - [FileField — File uploads](#filefield--file-uploads)
    - [Associated JS files](#associated-js-files)
    - [ColorField — Color picker](#colorfield--color-picker)
    - [SlugField — URL-friendly slug](#slugfield--url-friendly-slug)
    - [UUIDField](#uuidfield)
    - [JSONField — Textarea with JSON validation](#jsonfield--textarea-with-json-validation)
    - [IPAddressField — IP address](#ipaddressfield--ip-address)
    - [HiddenField — Hidden field](#hiddenfield--hidden-field)
  - [Field types summary](#field-types-summary)
  - [Database errors](#database-errors)
  - [Template rendering](#template-rendering)
    - [Full form](#full-form)
    - [Field by field](#field-by-field)
    - [Global errors](#global-errors)
    - [Field data as JSON](#field-data-as-json)
  - [Full example: signup with persistence](#full-example-signup-with-persistence)
    - [GET/POST handler](#getpost-handler)
  - [⚠️ Common pitfalls](#️-common-pitfalls)
    - [1. Template variable name collision](#1-template-variable-name-collision)
    - [2. Forgetting `mut` on form](#2-forgetting-mut-on-form)
    - [3. Comparing passwords after `is_valid()`](#3-comparing-passwords-after-is_valid)
  - [Next steps](#next-steps)

---

---

## Overview

[↑](#table-of-contents)

Runique provides a powerful form system inspired by Django. There are **two approaches**:

1. **Manual** — Define fields via the `RuniqueForm` trait.
2. **Automatic** — Derive a form from a `model!` schema with `#[form(...)]`.

Forms are automatically extracted from requests via the **Prisme** extractor, handle validation (including via the `validator` crate for emails/URLs), CSRF, Argon2 password hashing, and can be saved directly to the database.

---

[↑](#table-of-contents)

## Prisme extractor

`Prisme<T>` is an Axum extractor that orchestrates a full pipeline behind the scenes:

1. **Sentinel** — Verifies access rules (login, roles) via `GuardRules`.
2. **Aegis** — Single body extraction (multipart, urlencoded, json) normalized into a `HashMap`.
3. **CSRF Gate** — Verifies the CSRF token in parsed data.
4. **Construction** — Builds the form `T`, fills fields, and runs validation.

```rust
use runique::prelude::*;

pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            // Valid form → processing
        }
    }
    // ...
}
```

> **💡** The developer simply writes `Prisme(mut form)` — the entire security pipeline is transparent.

---

[↑](#table-of-contents)

## Manual approach: RuniqueForm trait

### Base structure

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
> **Equivalent without the macro (for reference)**

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

[↑](#table-of-contents)

### RuniqueForm trait methods

| Method                              | Role                                                            |
| ----------------------------------- | --------------------------------------------------------------- |
| `register_fields(form)`             | Declare the form fields                                         |
| `from_form(form)`                   | Build the instance from a `Forms`                               |
| `get_form()` / `get_form_mut()`     | Accessors for the internal `Forms`                              |
| `clean()`                           | Cross-field business logic (e.g. `pwd1 == pwd2`) — **optional** |
| `is_valid()`                        | Full pipeline: field validation → `clean()` → `finalize()`. **Note: is_valid() only validates if the form has received data (excluding CSRF), which prevents error messages from appearing on GET or initial display.** |
| `database_error(&err)`              | Inject a DB error on the correct field                          |
| `build(tera, csrf_token)`           | Build an empty form                                             |
| `build_with_data(data, tera, csrf)` | Build, fill, and validate                                       |

[↑](#table-of-contents)

### `is_valid()` validation pipeline

Calling `form.is_valid().await` triggers **3 steps in order**:

1. **Field validation** — Each field runs `validate()`: required, length, format (email via `validator`, URL via `validator`, JSON via `serde_json`, UUID via `uuid`, IP via `std::net::IpAddr`, …)
2. **`clean()`** — Custom business logic (passwords are still plain text at this step, which allows comparing `pwd1 == pwd2`)
3. **`finalize()`** — Final transformations (automatic Argon2 hashing for `Password` fields)

```rust
#[async_trait::async_trait]
impl RuniqueForm for RegisterForm {
    // ...

    async fn clean(&mut self) -> Result<(), StrMap> {
        let mdp1 = self.form.get_string("password");
        let mdp2 = self.form.get_string("password_confirm");

        if mdp1 != mdp2 {
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

> **⚠️ Important**: After is_valid(), Password fields are automatically hashed if the password hashing configuration is set to automatic. Otherwise, the developer must call the hash function within the business logic of save().
Use clean() for any comparison involving plaintext passwords.

---

[↑](#table-of-contents)

## Typed conversion helpers

Form values are stored as `String`. Instead of parsing manually, use the typed helpers on `Forms`:

### Direct conversions

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

[↑](#table-of-contents)

### Option conversions

```rust
form.get_option("bio")           // -> Option<String> (None if empty)
form.get_option_i32("age")       // -> Option<i32>
form.get_option_i64("score")     // -> Option<i64>
form.get_option_f64("note")      // -> Option<f64> (handles , → .)
form.get_option_bool("news")     // -> Option<bool>
```

[↑](#table-of-contents)

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

> **💡** Float helpers (`get_f32`, `get_f64`, `get_option_f64`) automatically convert commas to dots (`19,99` → `19.99`) for French locales.

---

---

[↑](#table-of-contents)

## Field types

### TextField — Text fields

`TextField` supports 6 special formats via the `SpecialFormat` enum:

```rust
// Plain text
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

**Builder options:**

```rust
TextField::text("name")
    .label("My field")              // Display label
    .placeholder("Enter...")        // Placeholder
    .required()                     // Required (default message)
    .min_length(3, "Too short")     // Min length with message
    .max_length(100, "Too long")    // Max length with message
    .readonly("Read-only")          // Read-only
    .disabled("Disabled")           // Disabled
```

**Automatic behavior per format:**

| Format     | Validation                 | Transformation                                           |
| ---------- | -------------------------- | -------------------------------------------------------- |
| `Email`    | `validator::ValidateEmail` | Lowercased                                               |
| `Url`      | `validator::ValidateUrl`   | —                                                        |
| `Password` | Standard                   | Argon2 hash in `finalize()`, value cleared on `render()` |
| `RichText` | Standard                   | XSS sanitization (`sanitize()`) before validation        |
| `Csrf`     | Session token              | —                                                        |

**Password utilities:**

```rust
// Hash manually
let hash = field.hash_password()?;

// Verify a password
let ok = TextField::verify_password("plain_pwd", "$argon2...");
```

> Automatic hashing detects if the value already starts with `$argon2` to avoid double hashing.

[↑](#table-of-contents)

### NumericField — Numeric fields

5 variants via the `NumericConfig` enum:

```rust
// Integer with bounds
form.field(
    &NumericField::integer("age")
        .label("Age")
        .min(0.0, "Min 0")
        .max(150.0, "Max 150"),
);

// Float number
form.field(&NumericField::float("price").label("Price"));

// Decimal with precision
form.field(
    &NumericField::decimal("amount")
        .label("Amount")
        .digits(2, 4),  // min 2, max 4 digits after the decimal separator
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

[↑](#table-of-contents)

### BooleanField — Checkboxes / Single radio

```rust
// Simple checkbox
form.field(
    &BooleanField::new("accept_terms")
        .label("I accept the terms")
        .required(),
);

// Single radio (yes/no)
form.field(&BooleanField::radio("newsletter").label("Newsletter"));

// Pre-checked
form.field(&BooleanField::new("remember_me").label("Remember me").checked());
```

[↑](#table-of-contents)

### ChoiceField — Select / Dropdown

```rust
use runique::forms::fields::choice::ChoiceOption;

let choices = vec![
    ChoiceOption::new("fr", "France"),
    ChoiceOption::new("be", "Belgium"),
    ChoiceOption::new("ch", "Switzerland"),
];

// Single select
form.field(
    &ChoiceField::new("country")
        .label("Country")
        .choices(choices.clone())
        .required(),
);

// Multiple select
form.field(
    &ChoiceField::new("languages")
        .label("Languages")
        .choices(choices)
        .multiple(),
);
```

> Validation automatically checks that the submitted value is among the declared choices.

[↑](#table-of-contents)

### RadioField — Radio buttons

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

[↑](#table-of-contents)

### CheckboxField — Multiple checkboxes

```rust
form.field(
    &CheckboxField::new("hobbies")
        .label("Hobbies")
        .choices(vec![
            ChoiceOption::new("sport", "Sports"),
            ChoiceOption::new("musique", "Music"),
            ChoiceOption::new("lecture", "Reading"),
        ]),
);
```

> Submitted values are in the form `"val1,val2,val3"`. Validation checks that each value exists in the choices.

### DateField, TimeField, DateTimeField — Date / Time

```rust
use chrono::NaiveDate;

// Date (format: YYYY-MM-DD)
form.field(
    &DateField::new("birthday")
        .label("Birth date")
        .min(NaiveDate::from_ymd_opt(1900, 1, 1).unwrap(), "Too old")
        .max(NaiveDate::from_ymd_opt(2010, 12, 31).unwrap(), "Too recent"),
);

// Time (format: HH:MM)
form.field(&TimeField::new("meeting_time").label("Meeting time"));

// Date + Time (format: YYYY-MM-DDTHH:MM)
form.field(&DateTimeField::new("event_start").label("Event start"));
```

[↑](#table-of-contents)

### DurationField — Duration

```rust
form.field(
    &DurationField::new("timeout")
        .label("Delay (seconds)")
        .min_seconds(60, "Minimum 1 minute")
        .max_seconds(3600, "Maximum 1 hour"),
);
```

[↑](#table-of-contents)

### FileField — File uploads

```rust
use runique::config::StaticConfig;

let config = StaticConfig::from_env();

// Image with full constraints
form.field(
    &FileField::image("avatar")
        .label("Profile picture")
        .upload_to(&config)
        .max_size_mb(5)
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp"]),
);

// Document
form.field(
    &FileField::document("cv")
        .label("Resume")
        .max_size_mb(10),
);

// Any file (multi-file)
form.field(
    &FileField::any("attachments")
        .label("Attachments")
        .max_files(5),
);
```

> **Security**: `.svg` files are **always rejected** by default (XSS risk). Image validation uses the `image` crate to check the real file format.

[↑](#table-of-contents)

### Associated JS files

```rust
fn register_fields(form: &mut Forms) {
    // ... fields ...
    form.add_js(&["js/my_script.js", "js/other.js"]);
}
```

JS files are automatically included in the form’s HTML rendering.

[↑](#table-of-contents)

### ColorField — Color picker

```rust
form.field(
    &ColorField::new("theme_color")
        .label("Theme color")
        .default_color("#3498db"),  // Validates #RGB or #RRGGBB format
);
```

[↑](#table-of-contents)

### SlugField — URL-friendly slug

```rust
form.field(
    &SlugField::new("slug")
        .label("Slug")
        .placeholder("my-url-article")
        .allow_unicode(),  // Optional: allow unicode characters
);
```

> Validation: letters, digits, hyphens, underscores only. Cannot start or end with a hyphen.

[↑](#table-of-contents)

### UUIDField

```rust
form.field(
    &UUIDField::new("external_id")
        .label("External ID")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
);
```

[↑](#table-of-contents)

### JSONField — Textarea with JSON validation

```rust
form.field(
    &JSONField::new("metadata")
        .label("Metadata")
        .placeholder(r#"{"key": "value"}"#)
        .rows(10),  // Number of textarea rows
);
```

[↑](#table-of-contents)

### IPAddressField — IP address

```rust
// IPv4 + IPv6
form.field(&IPAddressField::new("server_ip").label("Server IP"));

// IPv4 only
form.field(&IPAddressField::new("gateway").label("Gateway").ipv4_only());

// IPv6 only
form.field(&IPAddressField::new("ipv6").label("IPv6 address").ipv6_only());
```

---

[↑](#table-of-contents)

### HiddenField — Hidden field

An invisible field in the HTML form (`<input type="hidden">`). Two main uses: pass technical data without showing it to the user, or manually validate a CSRF token.

```rust
// Generic hidden field (e.g. linked entity ID)
form.field(
    &HiddenField::new("entity_id")
        .label("Entity ID"),
);

// Internal CSRF field (managed automatically by Runique — advanced use only)
form.field(&HiddenField::new_csrf());
```

> In standard Runique forms, CSRF is handled automatically via `{% csrf %}` in the template. You don't need `HiddenField::new_csrf()` unless you are building a fully custom form.

---

[↑](#table-of-contents)

## Field types summary

| Struct           | Constructors                                                           | Special validation                                  |
| ---------------- | ---------------------------------------------------------------------- | --------------------------------------------------- |
| `TextField`      | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL via `validator`, Argon2, XSS sanitization |
| `NumericField`   | `integer()`, `float()`, `decimal()`, `percent()`, `range()`            | Min/max bounds, decimal precision                   |
| `BooleanField`   | `new()`, `radio()`                                                     | Required = must be checked                          |
| `ChoiceField`    | `new()` + `.multiple()`                                                | Value must be in declared choices                   |
| `RadioField`     | `new()`                                                                | Value must be in declared choices                   |
| `CheckboxField`  | `new()`                                                                | All values must be in choices                       |
| `DateField`      | `new()`                                                                | `YYYY-MM-DD` format, min/max bounds                 |
| `TimeField`      | `new()`                                                                | `HH:MM` format, min/max bounds                      |
| `DateTimeField`  | `new()`                                                                | `YYYY-MM-DDTHH:MM` format, min/max bounds           |
| `DurationField`  | `new()`                                                                | Seconds, min/max bounds                             |
| `FileField`      | `image()`, `document()`, `any()`                                       | Extensions, size, dimensions, anti-SVG              |
| `ColorField`     | `new()`                                                                | `#RRGGBB` or `#RGB` format                          |
| `SlugField`      | `new()`                                                                | ASCII/unicode, no hyphen at start/end               |
| `UUIDField`      | `new()`                                                                | Valid UUID format                                   |
| `JSONField`      | `new()`                                                                | Valid JSON via `serde_json`                         |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()`                              | IPv4/IPv6 via `std::net::IpAddr`                    |
| `HiddenField`    | `new()`, `new_csrf()`                                                  | CSRF token validation if `name == "csrf_token"`     |

[↑](#table-of-contents)

## Database errors

The `database_error()` method automatically analyzes DB errors to attach the error to the correct field:

```rust
match form.save(&request.engine.db).await {
    Ok(_) => { /* success */ }
    Err(err) => {
        form.database_error(&err);
        // The error is set on the relevant field
    }
}
```

**Supported error formats:**

- **PostgreSQL**: `UNIQUE constraint`, `Key (field)=(value)`
- **SQLite**: `UNIQUE constraint failed: table.field`
- **MySQL**: `Duplicate entry ... for key 'table.field'`

If the field is identified, the error appears on that field (e.g. “This email is already used”). Otherwise, it is added to global errors.

---

[↑](#table-of-contents)

## Template rendering

### Full form

```html
<form method="post">
    {% form.inscription_form %}
    <button type="submit">Sign up</button>
</form>
```

Automatically renders: all fields, labels, validation errors, CSRF token, and JS scripts.

[↑](#table-of-contents)

### Field by field

```html
<form method="post">
    {% csrf %} <!-- included in the form, not needed manually -->
    <div class="row">
        <div class="col-6">{% form.inscription_form.username %}</div>
        <div class="col-6">{% form.inscription_form.email %}</div>
    </div>
    {% form.inscription_form.password %}
    <button type="submit">Sign up</button>
</form>
```

[↑](#table-of-contents)

### Global errors

```html
{% if inscription_form.errors %}
    <div class="alert alert-danger">
        {% for msg in inscription_form.errors %}
            <p>{{ msg }}</p>
        {% endfor %}
    </div>
{% endif %}
```

[↑](#table-of-contents)

### Field data as JSON

Forms automatically serialize `data`, `errors`, `errors`, `html`, `rendered_fields`, `fields` and `js_files`.

---

## Full example: signup with persistence

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
            // The password is already Argon2-hashed after is_valid()
            password: Set(self.form.get_string("password")),
            ..Default::default()
        };
        model.insert(db).await
    }
}
```

[↑](#table-of-contents)

### GET/POST handler

```rust
pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    let template = "profile/register_form.html";

    if request.is_get() {
        context_update!(request => {
            "title" => "Sign up",
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

[↑](#table-of-contents)

## ⚠️ Common pitfalls

### 1. Template variable name collision

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

[↑](#table-of-contents)

### 2. Forgetting `mut` on form

```rust
//  Cannot call is_valid()
Prisme(form): Prisme<MyForm>

//  Correct
Prisme(mut form): Prisme<MyForm>
```

[↑](#table-of-contents)

### 3. Comparing passwords after `is_valid()`

```rust

/// main.rs ->
/// with this configuration ->
password_init(PasswordConfig::auto_with(Manual::Argon2));
//  and is_valid(), passwords are hashed!
let mdp = form.get_form().get_string("password");
// mdp == "$argon2id$v=19$m=..." 😱

//  Compare in clean(), BEFORE finalization
async fn clean(&mut self) -> Result<(), StrMap> {
    let mdp1 = self.form.get_string("password");
    let mdp2 = self.form.get_string("password_confirm");
    if mdp1 != mdp2 { /* error */ }
    Ok(())
}
```

---

[↑](#table-of-contents)

## Next steps

← **[Routing](https://github.com/seb-alliot/runique/blob/main/docs/en/routing/04-routing.md)** | **[Templates](https://github.com/seb-alliot/runique/blob/main/docs/en/template/06-templates.md)** →

---
