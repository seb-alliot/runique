# Extraction de formulaire — `request.form()`

[← Formulaires](/docs/fr/formulaire)

---

`request.form()` est une méthode intégrée dans `Request` qui orchestre un pipeline complet en coulisses :

1. **Aegis** — Extraction unique du body (multipart, urlencoded, json) normalisée en `HashMap`, dans la limite de taille de la requête. Pour un multipart, aucun fichier n'est écrit avant un jeton CSRF valide (en-tête `X-CSRF-Token`, ou champ `csrf_token` placé avant les fichiers).
2. **CSRF** — Le pipeline note si le jeton est valide. Il ne rejette pas la requête pour autant : `request.form()` marque le formulaire invalide (`csrf.invalid_or_missing`) et `try_form` ne le valide pas.
3. **Construction** — Crée le formulaire `T` et remplit les champs. La validation se lance ensuite, dans `ValidationForm::try_form`.

```rust
use runique::prelude::*;

pub async fn inscription(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();
    if let Ok(validated) = ValidationForm::try_form(form, &request, "my_form").await {
        // Formulaire valide → traitement
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
    let form: RegisterForm = request.form();
    let template = "inscription_form.html";

    // GET (rien soumis) ou POST invalide → réafficher avec les erreurs
    let validated = match ValidationForm::try_form(form, &request, "my_form").await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "inscription_form" => &form });
            return request.render(template);
        }
    };

    // POST valide — enregistrer. `save` consomme le formulaire et le rend :
    // `Ok(form)` une fois écrit, `Err(form)` avec l'erreur DB déjà posée.
    match validated.save(&request.engine.db).await {
        Ok(_form) => {
            success!(request.notices => "Bienvenue !");
            Ok(Redirect::to("/").into_response())
        }
        Err(form) => {
            context_update!(request => { "inscription_form" => &form });
            request.render(template)
        }
    }
}
```

Points clés :

- `request.form()` renvoie un formulaire utilisable directement — pas de construction manuelle.
- `ValidationForm::try_form(form, &request, "my_form").await` dispatche sur la méthode HTTP (`allow_get`/`allow_post`) puis valide ; `Ok` prouve au niveau du type que le formulaire a été validé, `Err` renvoie le formulaire avec ses erreurs de champ à ré-afficher (rendues automatiquement par `{% form.inscription_form %}`).
- `validated.save(&request.engine.db).await` persiste l'entité et renvoie le formulaire : `Ok(form)` une fois écrit, `Err(form)` si la base refuse (contrainte unique, ligne absente). L'erreur est déjà sur ce formulaire.

---

← [**Formulaires**](/docs/fr/formulaire) | [**Trait RuniqueForm**](/docs/fr/formulaire/trait) →
