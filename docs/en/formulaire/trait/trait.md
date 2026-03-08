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
```

**Method reference:**

**`register_fields(form)`** — Declare the form fields.

**`from_form(form)`** — Build the instance from a `Forms`.

**`get_form()` / `get_form_mut()`** — Accessors for the internal `Forms`.

**`clean_field(name)`** *(optional)* — Per-field business validation. Returns `bool`. Called after `validate()` for each field.

**`clean()`** *(optional)* — Cross-field validation. Returns `Result<(), StrMap>`. Called once all fields are valid.

**`is_valid()`** — Orchestrates the full pipeline. Skipped if the form has not received data (prevents errors on GET).

**`database_error(&err)`** — Parses a DB error and attaches it to the correct field.

**`build(tera, csrf_token)`** — Build an empty form.

**`build_with_data(data, tera, csrf)`** — Build, fill, and validate.

---

## `is_valid()` validation pipeline

Calling `form.is_valid().await` triggers **4 steps in order**:

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

> **⚠️ Important**: After `is_valid()`, `Password` fields are **automatically hashed**.
> Use `clean()` for any password comparison — it is the only step where they are still readable.

---

← [**Prisme extractor**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/prisme/prisme.md) | [**Typed conversion helpers**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/helpers/helpers.md) →
