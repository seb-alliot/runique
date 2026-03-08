# Extracteur Prisme

[← Formulaires](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md)

---

`Prisme<T>` est un extracteur Axum qui orchestre un pipeline complet en coulisses :

1. **Sentinel** — Vérifie les règles d'accès (login, rôles) via `GuardRules`.
2. **Aegis** — Extraction unique du body (multipart, urlencoded, json) normalisée en `HashMap`.
3. **CSRF Gate** — Vérifie le token CSRF dans les données parsées.
4. **Construction** — Crée le formulaire `T`, remplit les champs et lance la validation.

```rust
use runique::prelude::*;

pub async fn inscription(
    mut request: Request,
    Prisme(mut form): Prisme<RegisterForm>,
) -> AppResult<Response> {
    if request.is_post() {
        if form.is_valid().await {
            // Formulaire valide → traitement
        }
    }
    // ...
}
```

> **💡** Le développeur écrit simplement `Prisme(mut form)` — tout le pipeline sécurité est transparent.

---

← [**Formulaires**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/05-forms.md) | [**Trait RuniqueForm**](https://github.com/seb-alliot/runique/blob/main/docs/fr/formulaire/trait/trait.md) →
