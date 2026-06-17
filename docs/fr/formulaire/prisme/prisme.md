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

## Exemple complet — afficher, valider, enregistrer

Un même handler gère l'affichage (GET) et la soumission (POST). `request.form()` construit le formulaire dans les deux cas : vide au GET, rempli depuis le body au POST.

```rust
use runique::prelude::*;

pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let mut form: RegisterForm = request.form();
    let template = "inscription_form.html";

    // GET — afficher le formulaire vide
    if request.is_get() {
        context_update!(request => { "inscription_form" => &form });
        return request.render(template);
    }

    // POST — valider puis enregistrer
    if request.is_post() && form.is_valid().await {
        match form.save(&request.engine.db).await {
            Ok(user) => {
                success!(request.notices => format!("Bienvenue {} !", user.username));
                return Ok(Redirect::to("/").into_response());
            }
            Err(err) => {
                // Erreur DB (ex. contrainte unique) reportée sur le formulaire
                form.database_error(&err);
            }
        }
    }

    // POST invalide ou erreur DB — réafficher avec les erreurs
    context_update!(request => { "inscription_form" => &form });
    request.render(template)
}
```

Points clés :

- `request.form()` renvoie un formulaire utilisable directement — pas de construction manuelle.
- `form.is_valid().await` agrège les erreurs de validation ; elles sont rendues automatiquement par `{{ form.inscription_form | form | safe }}` dans le template.
- `form.save(&request.engine.db).await` persiste l'entité et renvoie le modèle créé.
- `database_error(&err)` reporte une erreur DB (ex. email déjà pris) comme erreur de formulaire plutôt que comme 500.

---

← [**Formulaires**](/docs/fr/formulaire) | [**Trait RuniqueForm**](/docs/fr/formulaire/trait) →
