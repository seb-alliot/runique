# Extraction de formulaire — `request.form()`

[← Formulaires](/docs/fr/formulaire)

---

`request.form()` est une méthode intégrée dans `Request` qui orchestre un pipeline complet en coulisses :

1. **Sentinel** — Vérifie les règles d'accès (login, rôles) via `GuardRules`.
2. **Aegis** — Extraction unique du body (multipart, urlencoded, json) normalisée en `HashMap`.
3. **CSRF Gate** — Vérifie le token CSRF dans les données parsées.
4. **Construction** — Crée le formulaire `T`, remplit les champs et lance la validation.

```rust
use runique::prelude::*;

pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    if request.is_post() {
        if form.is_valid().await {
            // Formulaire valide → traitement
        }
    }
    // ...
}
```

> **💡** Le développeur appelle simplement `request.form()` — tout le pipeline sécurité est transparent.

---

← [**Formulaires**](/docs/fr/formulaire) | [**Trait RuniqueForm**](/docs/fr/formulaire/trait) →
