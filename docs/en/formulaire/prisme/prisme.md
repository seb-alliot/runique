# Form extraction — `request.form()`

[← Forms](/docs/en/formulaire)

---

`request.form()` is a method built into `Request` that orchestrates a full pipeline behind the scenes:

1. **Aegis** — Single body extraction (multipart, urlencoded, json) normalized into a `HashMap`, within the request size limit. For a multipart body, no file is written before a valid CSRF token (`X-CSRF-Token` header, or a `csrf_token` field placed before the files).
2. **CSRF** — The pipeline records whether the token is valid. It does not reject the request for that: `request.form()` marks the form invalid (`csrf.invalid_or_missing`) and `try_form` will not validate it.
3. **Construction** — Builds the form `T` and fills its fields. Validation runs afterwards, in `ValidationForm::try_form`.

```rust
use runique::prelude::*;

pub async fn register(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();
    if let Ok(validated) = ValidationForm::try_form(form, &request, "my_form").await {
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
    let validated = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "register_form" => &form });
            return request.render(template);
        }
    };

    // Valid POST — save. `save` consumes the form and gives it back:
    // `Ok(form)` once written, `Err(form)` with the DB error already attached.
    match validated.save(&request.engine.db).await {
        Ok(_form) => {
            success!(request.notices => "Welcome!");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => { "register_form" => &form });
            request.render(template)
        }
    }
}
```

Key points:

- `request.form()` returns a ready-to-use form — no manual construction.
- `ValidationForm::try_form(form, &request, "my_form").await` dispatches on the HTTP method (`allow_get`/`allow_post`) then validates; `Ok` proves at the type level the form was validated, `Err` returns the form with its field errors to re-render (rendered automatically by `{% form.register_form %}`).
- `validated.save(&request.engine.db).await` persists the entity and returns the form: `Ok(form)` once written, `Err(form)` if the database refuses (unique constraint, missing row). The error is already on that form.

---

← [**Forms**](/docs/en/formulaire) | [**RuniqueForm trait**](/docs/en/formulaire/trait) →
