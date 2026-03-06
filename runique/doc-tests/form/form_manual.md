# Utilisation manuelle du formulaire

```rust,ignore
use runique::prelude::*;
use runique::forms::{Forms, fields::text::TextField, fields::number::NumericField};

async fn register_handler(ctx: Request, Form(data): Form<HashMap<String, String>>) -> Response {
    let csrf_token = ctx.csrf_token();

    // Création du formulaire
    let mut form = Forms::new(&csrf_token);
    form.set_renderer(ctx.engine.renderer());

    // Ajout des champs
    form.field(&TextField::text("username").label("Nom d'utilisateur").required(true, None));
    form.field(&TextField::email("email").label("Email").required(true, None));
    form.field(&NumericField::integer("age").label("Âge"));
    form.field(&TextField::password("password").label("Mot de passe").required(true, None));

    // Si POST : remplissage et validation
    if ctx.is_post() {
        form.fill(&data);

        if form.is_valid().unwrap_or(false) {
            let username = form.get_string("username");
            let email    = form.get_string("email");
            let age      = form.get_i32("age");

            // Logique métier...

            return Redirect::to("/success").into_response();
        }
    }

    // Rendu du formulaire (GET ou POST invalide)
    let mut context = ctx.context.clone();
    context.insert("form", &form);
    ctx.render("register.html", &context)
}
```

## Récupération des valeurs

```rust,ignore
// Chaîne de caractères
let username: String = form.get_string("username");

// Entiers et décimaux
let age: i32   = form.get_i32("age");
let score: f64 = form.get_f64("score");

// Booléen (true si "true", "1" ou "on")
let active: bool = form.get_bool("active");

// Optionnels (None si champ vide)
let bio: Option<String> = form.get_option("bio");

// Dates
let birthday: chrono::NaiveDate = form.get_naive_date("birthday");
```
