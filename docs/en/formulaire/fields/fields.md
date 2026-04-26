# Field types

[← Typed conversion helpers](/docs/en/formulaire/helpers)

---

## TextField — Text fields

`TextField` supports 6 special formats via the `SpecialFormat` enum:

```rust
// Plain text
form.field(&TextField::text("username").label("Name").required());

// Email — validated via `validator::ValidateEmail`
form.field(&TextField::email("email").label("Email").required());

// URL — validated via `validator::ValidateUrl`
form.field(&TextField::url("website").label("Website"));

// Registration / password change form — automatic hashing in finalize()
form.field(
    &TextField::password("password")
        .label("Password")
        .required()
        .min_length(8, "Min 8 characters"),
);

// Login form — .no_hash() is required
// Without .no_hash(), finalize() hashes the submitted password before comparison
// → verify(hash, stored_hash) always fails silently
form.field(
    &TextField::password("password")
        .label("Password")
        .no_hash()
        .required(),
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

| Format     | Validation                 | Transformation                                                                                 |
| ---------- | -------------------------- | ---------------------------------------------------------------------------------------------- |
| `Email`    | `validator::ValidateEmail` | Lowercased                                                                                     |
| `Url`      | `validator::ValidateUrl`   | —                                                                                              |
| `Password` | Standard                   | Auto hash in `finalize()` if config is `Auto` and no `.no_hash()`, value cleared on `render()` |
| `RichText` | Standard                   | XSS sanitization (`sanitize()`) before validation                                              |
| `Csrf`     | Session token              | —                                                                                              |

**Password utilities:**

Hashing and verification are delegated to `PasswordConfig`, initialized at startup via `password_init()`:

```rust
use runique::prelude::{hash, verify};

// Hash manually (e.g. account creation outside a form)
let hashed = hash("my_password")?;

// Verify a plain password against a stored hash (e.g. login)
let ok = verify("plain_pwd", &user.password_hash);
if !ok {
    // incorrect password
}
```

> Automatic hashing in `finalize()` detects if the value already starts with `$argon2` to avoid double hashing. In a **login** form, do not rely on `is_valid()` to check the password — fetch the user from the DB first, then call `verify()` manually.

---

## NumericField — Numeric fields

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

**Options:** `.min(val, msg)`, `.max(val, msg)`, `.step(val)` (`range` type only), `.digits(min, max)`, `.label(l)`, `.placeholder(p)`

---

## BooleanField — Checkboxes / Single radio

```rust
// Simple checkbox
form.field(
    &BooleanField::new("accept_terms")
        .label("I accept the terms")
        .required(),
);
```

> **Note on `.required()`**: On a `BooleanField`, `.required()` means the field is mandatory at the database level (`NOT NULL`). It does not force the user to check the box to validate the form (an unchecked checkbox sends `false`, which is a valid value for a non-null boolean). To force acceptance (e.g. Terms), use a custom validation or check the value manually.

```rust
// Single radio (yes/no)
form.field(&BooleanField::radio("newsletter").label("Newsletter"));

// Pre-checked
form.field(&BooleanField::new("remember_me").label("Remember me").checked());
```

---

## ChoiceField — Select / Dropdown

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

---

## RadioField — Radio buttons

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

---

## CheckboxField — Multiple checkboxes

```rust
form.field(
    &CheckboxField::new("hobbies")
        .label("Hobbies")
        .choices(vec![
            ChoiceOption::new("sport", "Sports"),
            ChoiceOption::new("music", "Music"),
            ChoiceOption::new("reading", "Reading"),
        ]),
);
```

> Submitted values are in the form `"val1,val2,val3"`. Validation checks that each value exists in the choices.

---

## DateField, TimeField, DateTimeField — Date / Time

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

---

## DurationField — Duration

```rust
form.field(
    &DurationField::new("timeout")
        .label("Delay (seconds)")
        .min_seconds(60, "Minimum 1 minute")
        .max_seconds(3600, "Maximum 1 hour"),
);
```

---

## FileField — File uploads

```rust
// Image with full constraints — explicit directory
form.field(
    &FileField::image("avatar")
        .label("Profile picture")
        .upload_to("uploads/avatars")   // → uploads/avatars/
        .max_size(FileSize::mb(5))
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp", "avif"]),
);

// Image — automatic directory from MEDIA_ROOT (.env)
// Files go to {MEDIA_ROOT}/{field_name}/  e.g. media/photo/
form.field(
    &FileField::image("photo")
        .label("Photo")
        .upload_to_env()
        .max_size(FileSize::mb(5)),
);

// Without upload_to — files stored directly in MEDIA_ROOT
form.field(
    &FileField::image("image")
        .label("Image")
        .max_size(FileSize::mb(5)),
);

// Document
form.field(
    &FileField::document("cv")
        .label("Resume")
        .upload_to("uploads/cv")
        .max_size(FileSize::mb(10)),
);

// Any file (multi-file)
form.field(
    &FileField::any("attachments")
        .label("Attachments")
        .max_files(5),
);
```

> **Note:** `.max_size()` is a form-level validation only. If the field is defined in a `model!{}` with `max_size(5 MB)`, the model limit and the form limit are two independent values — they are not automatically synchronized. For a manual form, they must be aligned by hand.

**File destination:**

| Method | Destination |
| --- | --- |
| `.upload_to("uploads/images")` | `uploads/images/` (exact path) |
| `.upload_to_env()` | `{MEDIA_ROOT}/{field_name}/` (from `.env`) |
| *(none)* | `MEDIA_ROOT` directly (no subdirectory) |

The move to the final destination happens in `finalize()`, **only if validation passes**. The directory is created automatically if it does not already exist.

> **Security**: `.svg` files are **always rejected** by default (XSS risk). Image validation uses the `image` crate to check the real file format. Empty submissions (no file selected) are handled correctly — the `required` constraint works as expected.

### Associated JS files

```rust
fn register_fields(form: &mut Forms) {
    // ... fields ...
    form.add_js(&["js/my_script.js", "js/other.js"]);
}
```

JS files are automatically included in the form's HTML rendering.

---

## ColorField — Color picker

```rust
form.field(
    &ColorField::new("theme_color")
        .label("Theme color")
        .default_color("#3498db"),  // Validates #RGB or #RRGGBB format
);
```

---

## SlugField — URL-friendly slug

```rust
form.field(
    &SlugField::new("slug")
        .label("Slug")
        .placeholder("my-url-article")
        .allow_unicode(),  // Optional: allow unicode characters
);
```

> Validation: letters, digits, hyphens, underscores only. Cannot start or end with a hyphen.

---

## UUIDField

```rust
form.field(
    &UUIDField::new("external_id")
        .label("External ID")
        .placeholder("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"),
);
```

---

## JSONField — Textarea with JSON validation

```rust
form.field(
    &JSONField::new("metadata")
        .label("Metadata")
        .placeholder(r#"{"key": "value"}"#)
        .rows(10),  // Number of textarea rows
);
```

---

## IPAddressField — IP address

```rust
// IPv4 + IPv6
form.field(&IPAddressField::new("server_ip").label("Server IP"));

// IPv4 only
form.field(&IPAddressField::new("gateway").label("Gateway").ipv4_only());

// IPv6 only
form.field(&IPAddressField::new("ipv6").label("IPv6 address").ipv6_only());
```

---

## HiddenField — Hidden field

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

## Field types summary

| Struct           | Constructors                                                           | Special validation                                  |
| ---------------- | ---------------------------------------------------------------------- | --------------------------------------------------- |
| `TextField`      | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL via `validator`, Argon2, XSS sanitization, `.rows(n)` |
| `NumericField`   | `integer()`, `float()`, `decimal()`, `percent()`, `range()`            | Min/max bounds, decimal precision, `.step(n)` (`range` only)   |
| `BooleanField`   | `new()`, `radio()`                                                     | Required = NOT NULL in database                                |
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
| `JSONField`      | `new()`                                                                | Valid JSON via `serde_json`, `.rows(n)`             |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()`                              | IPv4/IPv6 via `std::net::IpAddr`                    |
| `HiddenField`    | `new()`, `new_csrf()`                                                  | CSRF token validation if `name == "csrf_token"`     |

---

← [**Typed conversion helpers**](/docs/en/formulaire/helpers) | [**Database errors**](/docs/en/formulaire/errors) →
