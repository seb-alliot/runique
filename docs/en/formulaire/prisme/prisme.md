# Form extraction — `request.form()`

[← Forms](/docs/en/formulaire)

---

`request.form()` is a method built into `Request` that orchestrates a full pipeline behind the scenes:

1. **Sentinel** — Verifies access rules (login, roles) via `GuardRules`.
2. **Aegis** — Single body extraction (multipart, urlencoded, json) normalized into a `HashMap`.
3. **CSRF Gate** — Verifies the CSRF token in parsed data.
4. **Construction** — Builds the form `T`, fills fields, and runs validation.

```rust
use runique::prelude::*;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if request.is_post() {
        if form.is_valid().await {
            // Valid form → processing
        }
    }
    // ...
}
```

> **💡** The developer simply calls `request.form()` — the entire security pipeline is transparent.

---

## Full example — display, validate, save

A single handler serves both display (GET) and submission (POST). `request.form()` builds the form in both cases: empty on GET, filled from the body on POST.

```rust
use runique::prelude::*;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    let template = "register_form.html";

    // GET — render the empty form
    if request.is_get() {
        context_update!(request => { "register_form" => &form });
        return request.render(template);
    }

    // POST — validate then save
    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                success!(request.notices => format!("Welcome {} !", user.username));
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                // DB error (e.g. unique constraint) reported on the form
                form.database_error(&err);
            }
        }
    }

    // Invalid POST or DB error — re-render with errors
    context_update!(request => { "register_form" => &form });
    request.render(template)
}
```

Key points:

- `request.form()` returns a ready-to-use form — no manual construction.
- `form.is_valid().await` aggregates validation errors; they are rendered automatically by `{{ form.register_form | form | safe }}` in the template.
- `form.save(&request.engine.db).await` persists the entity and returns the created model.
- `database_error(&err)` reports a DB error (e.g. email already taken) as a form error rather than a 500.

---

← [**Forms**](/docs/en/formulaire) | [**RuniqueForm trait**](/docs/en/formulaire/trait) →
