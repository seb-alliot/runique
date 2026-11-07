# Extraction de formulaire — `request.form("nom")`

[← Formulaires](/docs/fr/formulaire)

---

`request.form("nom")` est une méthode intégrée dans `Request` qui orchestre un pipeline complet en coulisses :

1. **Aegis** — Extraction unique du body (multipart, urlencoded, json) normalisée en `HashMap`, dans la limite de taille de la requête. Pour un multipart, aucun fichier n'est écrit avant un jeton CSRF valide (en-tête `X-CSRF-Token`, ou champ `csrf_token` placé avant les fichiers).
2. **CSRF** — Le pipeline note si le jeton du champ `csrf_token` est valide. Il ne rejette pas la requête pour autant : `request.form("nom")` marque le formulaire invalide (`csrf.invalid_or_missing`) et `try_form` ne le valide pas. En amont, le middleware CSRF a déjà refusé en 403 un en-tête `X-CSRF-Token` faux, un corps qui n'est pas un formulaire (JSON…) sans en-tête, et un formulaire posté depuis un autre site.
3. **Construction** — Crée le formulaire `T` et remplit les champs. La validation se lance ensuite, dans `ValidationForm::try_form`, et seulement si la requête renvoie le nom donné au formulaire.

```rust
use runique::prelude::*;

pub async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");
    if let Ok(validated) = ValidationForm::try_form(form, &request).await {
        // Formulaire valide → traitement
    }
    // ...
}
```

> **💡** Le développeur appelle simplement `request.form("nom")` — tout le pipeline sécurité est transparent.

---

## Exemple complet — afficher, valider, enregistrer

Un même handler gère l'affichage (GET) et la soumission (POST). `request.form("nom")` construit le formulaire dans les deux cas : vide au GET, rempli depuis le body au POST.

```rust
use runique::prelude::*;

pub async fn signup(mut request: Request) -> AppResult<Response> {
    let form: SignupForm = request.form("signup");
    let template = "signup.html";

    // GET (rien soumis) ou POST invalide → réafficher avec les erreurs
    let validated = match ValidationForm::try_form(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            context_update!(request => { "signup_form" => &form });
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
            context_update!(request => { "signup_form" => &form });
            request.render(template)
        }
    }
}
```

Points clés :

- `request.form("nom")` renvoie un formulaire utilisable directement — pas de construction manuelle.
- `ValidationForm::try_form(form, &request).await` vérifie que la requête renvoie ce nom, dispatche sur la méthode HTTP (`allow_get`/`allow_post`) puis valide ; `Ok` prouve au niveau du type que le formulaire a été validé, `Err` renvoie le formulaire avec ses erreurs de champ à ré-afficher (rendues automatiquement par `{% form.signup_form %}`).
- `validated.save(&request.engine.db).await` persiste l'entité et renvoie le formulaire : `Ok(form)` une fois écrit, `Err(form)` si la base refuse (contrainte unique, ligne absente). L'erreur est déjà sur ce formulaire.

---

← [**Formulaires**](/docs/fr/formulaire) | [**Trait RuniqueForm**](/docs/fr/formulaire/trait) →
