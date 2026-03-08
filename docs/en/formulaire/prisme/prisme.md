# Prisme extractor

[← Forms](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md)

---

`Prisme<T>` is an Axum extractor that orchestrates a full pipeline behind the scenes:

1. **Sentinel** — Verifies access rules (login, roles) via `GuardRules`.
2. **Aegis** — Single body extraction (multipart, urlencoded, json) normalized into a `HashMap`.
3. **CSRF Gate** — Verifies the CSRF token in parsed data.
4. **Construction** — Builds the form `T`, fills fields, and runs validation.

```rust
use runique::prelude::*;

pub async fn register(
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

← [**Forms**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/05-forms.md) | [**RuniqueForm trait**](https://github.com/seb-alliot/runique/blob/main/docs/en/formulaire/trait/trait.md) →
