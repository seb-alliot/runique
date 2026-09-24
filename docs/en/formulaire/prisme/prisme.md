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
    let form: RegisterForm = request.form();
    if let Ok(validated) = ValidationForm::try_new(form, &request).await {
        // Valid form → processing
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
    let form: RegisterForm = request.form();
    let template = "register_form.html";

    // GET (nothing submitted) or invalid POST — re-render with errors
    let mut validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "register_form" => &form });
            return request.render(template);
        }
    };

    // Valid POST — save
    match validated.save(&request.engine.db).await {
        Ok(user) => {
            success!(request.notices => format!("Welcome {} !", user.username));
            return Ok(Redirect::to("/").into_response());
        }
        Err(err) => {
            // DB error (e.g. unique constraint) reported on the form
            validated.database_error(&err);
        }
    }

    context_update!(request => { "register_form" => &*validated });
    request.render(template)
}
```

Key points:

- `request.form()` returns a ready-to-use form — no manual construction.
- `ValidationForm::try_new(form, &request).await` dispatches on the HTTP method (`allow_get`/`allow_post`) then validates; `Ok` proves at the type level the form was validated, `Err` returns the form with its field errors to re-render (rendered automatically by `{{ form.register_form | form }}`).
- `validated.save(&request.engine.db).await` persists the entity and returns the created model.
- `database_error(&err)` reports a DB error (e.g. email already taken) as a form error rather than a 500 — callable directly on `ValidationForm` without going through `into_form()`.

---

← [**Forms**](/docs/en/formulaire) | [**RuniqueForm trait**](/docs/en/formulaire/trait) →
