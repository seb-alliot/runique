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

← [**Forms**](/docs/en/formulaire) | [**RuniqueForm trait**](/docs/en/formulaire/trait) →
