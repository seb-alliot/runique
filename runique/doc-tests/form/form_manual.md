# Utilisation manuelle du formulaire

```rust,ignore
use runique::prelude::*;
use runique::forms::{Forms, fields::text::TextField, fields::number::NumericField};

pub struct RegisterForm {
    pub form: Forms,
}

impl RuniqueForm for RegisterForm {
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username").label("Nom d'utilisateur").required());
        form.field(&TextField::email("email").label("Email").required());
        form.field(&NumericField::integer("age").label("Âge"));
        form.field(&TextField::password("password").label("Mot de passe").required());
    }
    impl_form_access!();
}

async fn register_handler(mut request: Request) -> AppResult<Response> {
    let form: RegisterForm = request.form();

    let validated = match ValidationForm::try_new(form, &request).await {
        Ok(validated) => validated,
        Err(form) => {
            // GET (rien soumis) ou soumission invalide — ré-afficher avec les erreurs
            context_update!(request => { "form" => &form });
            return request.render("register.html");
        }
    };

    let username = validated.cleaned_string("username");
    let email = validated.cleaned_string("email");
    let age = validated.cleaned_i32("age");

    // Logique métier...

    Ok(Redirect::to("/success").into_response())
}
```

`request.form()` construit le formulaire, extrait le body et vérifie le CSRF en coulisses — pas de `Forms::new(&csrf_token)` ni de `set_renderer()` manuel. `ValidationForm::try_new(form, &request)` remplace le `if ctx.is_post() { form.fill(&data); if form.is_valid()... }` : il dispatche lui-même sur la méthode HTTP (`allow_get`/`allow_post`) puis valide.

## Récupération des valeurs

Les accesseurs `cleaned_*` renvoient toujours un `Option<T>` (le champ peut être absent, vide, ou invalide) :

```rust,ignore
// Chaîne de caractères
let username: Option<String> = form.cleaned_string("username");

// Entiers et décimaux
let age: Option<i32> = form.cleaned_i32("age");
let score: Option<f64> = form.cleaned_f64("score");

// Booléen
let active: Option<bool> = form.cleaned_bool("active");

// Dates
let birthday: Option<chrono::NaiveDate> = form.cleaned_naive_date("birthday");
```
