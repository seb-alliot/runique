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

// Phone — digits/spaces/dashes/parentheses, leading '+' only, 7 to 15 digits
form.field(&TextField::phone("phone").label("Phone"));
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
| `Phone`    | Allowed chars (digits/spaces/`-`/`()`), leading `+` only, 7 to 15 digits | — |

**Password utilities:**

Hashing and verification are delegated to `PasswordConfig`, initialized at startup via `password_init()`:

```rust
use runique::prelude::{hash, verify};

// Hash manually (e.g. account creation outside a form)
let hashed = hash("my_password")?;

// Verify a plain password against a stored hash (e.g. login)
let ok = verify("plain_pwd", &user.password);
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

> **Note on `.required()`**: on a `BooleanField`, `.required()` means **"must be checked"** (terms, consent): an unchecked box is an error. A box always answers (`true` or `false`), so a NOT NULL column needs no `required`: in the DSL, `active: bool` is enough, and `terms: bool [required]` forces the check. `required` with `default: true` is a compile error: a box checked in advance isn't consent. The box only accepts the value it sends (`"true"`, or the one set with `.checked_value(..)`): any other value is an error.

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

## FileField and MediaField — File uploads

Two fields share the same upload mechanism (staging folder, move between the write and the commit, size, count, destination):

- **`MediaField`** — images, audio and video: `MediaField::image()`, `::audio()`, `::video()`. The **content** of each file is checked (its signature), not only its extension: a text file renamed `.mp4` is refused.
- **`FileField`** — documents and other files: `FileField::document()`, `::any()`. Only the extension, size and count are checked.

| Constructor | Extensions accepted by default | Content checked |
| --- | --- | --- |
| `MediaField::image` | jpg, jpeg, png, gif, webp, avif | signature + dimensions (`max_dimensions`) |
| `MediaField::audio` | mp3, ogg, oga, wav, m4a, flac, aac | signature (MP3, Ogg, WAV, FLAC, MP4/M4A, AAC) |
| `MediaField::video` | mp4, webm, mov, ogv | signature (MP4/MOV, WebM, Ogg) |
| `FileField::document` | pdf, doc, docx, txt, odt | no |
| `FileField::any` | all but active extensions (`.svg`, `.html`, `.js`…) | no |

The default maximum size is 10 MB for all; for audio and video, set `.max_size()`. The request body's global limit (`RUNIQUE_MAX_UPLOAD_MB`) applies too.

```rust
// Image with full constraints — explicit directory
form.field(
    &MediaField::image("avatar")
        .label("Profile picture")
        .upload_to("uploads/avatars")   // → {MEDIA_ROOT}/uploads/avatars/
        .max_size(FileSize::mb(5))
        .max_files(1)
        .max_dimensions(1920, 1080)
        .allowed_extensions(vec!["png", "jpg", "jpeg", "webp", "avif"]),
);

// Image — automatic directory from MEDIA_ROOT (.env)
// Files go to {MEDIA_ROOT}/{field_name}/  e.g. media/photo/
form.field(
    &MediaField::image("photo")
        .label("Photo")
        .upload_to_env()
        .max_size(FileSize::mb(5)),
);

// Without upload_to — files stored directly in MEDIA_ROOT
form.field(
    &MediaField::image("image")
        .label("Image")
        .max_size(FileSize::mb(5)),
);

// Audio and video: the content is checked as for an image
form.field(
    &MediaField::audio("podcast")
        .label("Episode")
        .upload_to("podcasts")
        .max_size(FileSize::mb(50)),
);
form.field(
    &MediaField::video("trailer")
        .label("Trailer")
        .upload_to("videos")
        .max_size(FileSize::mb(200)),
);

// Document
form.field(
    &FileField::document("cv")
        .label("Resume")
        .upload_to("uploads/cv")
        .max_size(FileSize::mb(10)),
);

// Any extension except those a browser runs (.svg, .html, .js…). Content is not checked:
// what the file is, and how it is served, is the application's responsibility.
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
| `.upload_to("uploads/images")` | `{MEDIA_ROOT}/uploads/images/` (relative to `MEDIA_ROOT`) |
| `.upload_to_env()` | `{MEDIA_ROOT}/{field_name}/` |
| *(none)* | `MEDIA_ROOT` directly (no subdirectory) |

`finalize()` prepares the relative path but keeps the file in its temporary staging directory, under `MEDIA_ROOT`, named `{uuid}.extension`. With `ValidationForm::save`, Runique writes the row, moves the file to its destination, then commits the transaction. Until the row is committed no one else sees it: no reader meets a path whose file isn't in place yet. A refused write (value already taken, foreign key…) leaves the file in staging, removed at the end of the request; only a failing commit itself takes back the file already moved. The destination must be relative to `MEDIA_ROOT` and cannot contain `.` or `..` components, Windows separators, or a volume prefix. The directory is created automatically if it does not already exist.

**The replaced file.** Runique never deletes the file a new upload replaces: you may still need it (history, archive). To delete it, read the old path before the `save`, then remove it once the write is done:

```rust
let old = profile::Entity::find_by_id(id).one(&request.engine.db).await?.and_then(|p| p.avatar);
match form.save(&request.engine.db).await {
    Ok(form) => {
        let new = form.forms().fields["avatar"].value().to_string();
        if let Some(old) = old.filter(|o| *o != new) {
            let _ = tokio::fs::remove_file(std::path::Path::new(&runique::config::static_files::resolve_media_root()).join(old)).await;
        }
        Ok(Redirect::to("/profile").into_response())
    }
    Err(form) => { /* the form with its errors */ }
}
```

**Without `save`.** A form without a model (a plain file upload) or a hand-written save handles the file itself: `form.forms_mut().publish_uploads().await` inside your transaction, after the write and before the `commit()`, then `complete_uploads().await` once committed, or `rollback_uploads().await` if the commit fails. With no database write, `publish_uploads()` then `complete_uploads()` is enough.

**Inside the handler's transaction** ([`request.db_atomic()`](/docs/en/orm/advanced)), the file is moved at the `save`. If the handler leaves without `commit()`, the row is rolled back but the file stays on disk, orphaned.

> **Current limit**: the disk takes no part in the transaction. A hard stop of the process between moving the file and the database commit, or a commit failure that can't be caught, can leave an orphan file under `MEDIA_ROOT` — never a row pointing at a missing file.

> **Security**: extensions a browser opens as a page of the site are **always rejected: `.svg`, `.svgz`, `.html`, `.htm`, `.xhtml`, `.xht`, `.shtml`, `.xml`, `.js`, `.mjs`** (XSS risk), whatever the field's list. `FileField::any` accepts every other extension and does not read the content: the security of that field is the application's responsibility. For a `MediaField`, validation checks the real format by reading the file's signature (not just the extension): image, audio or video. With `max_dimensions`, the `image` crate measures the file: if the measurement fails, the file is refused, for every format. Without that measurement, the width and height limits would not apply. Empty submissions (no file selected) are handled correctly — the `required` constraint works as expected.

### Associated JS files

```rust
fn register_fields(form: &mut Forms) {
    // ... fields ...
    form.add_js(&["js/my_script.js", "js/other.js"]);
}
```

JS files are automatically included in the form's HTML rendering, with the request's CSP nonce and their SRI hash.

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

An invisible field in the HTML form (`<input type="hidden">`). Main use: pass technical data without showing it to the user.

```rust
// Generic hidden field (e.g. linked entity ID)
form.field(
    &HiddenField::new("entity_id")
        .label("Entity ID"),
);
```

> The CSRF field is **emitted naturally** by every `RuniqueForm` (`req.form("name")` / `form_fields.html` / `{% form.xxx %}`) — no need for `HiddenField::new_csrf()` or `{% csrf %}` in the template. `{% csrf %}` is only for a hand-written HTML `<form>`, outside `RuniqueForm`.

---

## Field types summary

| Struct           | Constructors                                                           | Special validation                                  |
| ---------------- | ---------------------------------------------------------------------- | --------------------------------------------------- |
| `TextField`      | `text()`, `email()`, `url()`, `password()`, `textarea()`, `richtext()` | Email/URL via `validator`, Argon2, XSS sanitization, `.rows(n)` |
| `NumericField`   | `integer()`, `float()`, `decimal()`, `percent()`, `range()`            | Min/max bounds, decimal precision, `.step(n)` (`range` only)   |
| `BooleanField`   | `new()`, `radio()`                                                     | `required` = must be checked; only its value is accepted       |
| `ChoiceField`    | `new()` + `.multiple()`                                                | Value must be in declared choices                   |
| `RadioField`     | `new()`                                                                | Value must be in declared choices                   |
| `CheckboxField`  | `new()`                                                                | All values must be in choices                       |
| `DateField`      | `new()`                                                                | `YYYY-MM-DD` format, min/max bounds                 |
| `TimeField`      | `new()`                                                                | `HH:MM` format, min/max bounds                      |
| `DateTimeField`  | `new()`                                                                | `YYYY-MM-DDTHH:MM` format, min/max bounds           |
| `DurationField`  | `new()`                                                                | Seconds, min/max bounds                             |
| `MediaField`     | `image()`, `audio()`, `video()`                                        | Extensions, size, content signature, dimensions (image), anti-SVG |
| `FileField`      | `document()`, `any()`                                                  | Extensions, size, anti-SVG                          |
| `ColorField`     | `new()`                                                                | `#RRGGBB` or `#RGB` format                          |
| `SlugField`      | `new()`                                                                | ASCII/unicode, no hyphen at start/end               |
| `UUIDField`      | `new()`                                                                | Valid UUID format                                   |
| `JSONField`      | `new()`                                                                | Valid JSON via `serde_json`, `.rows(n)`             |
| `IPAddressField` | `new()` + `.ipv4_only()` / `.ipv6_only()`                              | IPv4/IPv6 via `std::net::IpAddr`                    |
| `HiddenField`    | `new()`, `new_csrf()`                                                  | CSRF token validation if `name == "csrf_token"`     |

---

← [**Typed conversion helpers**](/docs/en/formulaire/helpers) | [**Database errors**](/docs/en/formulaire/errors) →
