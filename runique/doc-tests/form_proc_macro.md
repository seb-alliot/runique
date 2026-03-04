# Formulaire avec `impl_form_access!`

La macro `impl_form_access!` génère automatiquement les méthodes d'accès requises par le trait `RuniqueForm`.

```rust,ignore
use runique::prelude::*;
use runique::forms::{RuniqueForm, Forms};
use runique::forms::fields::text::TextField;
use runique::forms::fields::number::NumericField;
use async_trait::async_trait;

pub struct RegisterForm {
    form: Forms,
}

#[async_trait]
impl RuniqueForm for RegisterForm {
    // Génère automatiquement : from_form, get_form, get_form_mut
    impl_form_access!();

    fn register_fields(form: &mut Forms) {
        form.field(&TextField::text("username")
            .label("Nom d'utilisateur")
            .min_length(3, "Minimum 3 caractères")
            .max_length(50, "Maximum 50 caractères")
            .required(true, None));

        form.field(&TextField::email("email")
            .label("Email")
            .required(true, None));

        form.field(&TextField::password("password")
            .label("Mot de passe")
            .min_length(8, "Minimum 8 caractères")
            .required(true, None));

        form.field(&NumericField::integer("age")
            .label("Âge")
            .min_value(13, "Vous devez avoir au moins 13 ans"));
    }
}

// Utilisation dans un handler
async fn register_handler(ctx: Request, Form(data): Form<HashMap<String, String>>) -> Response {
    let csrf = ctx.csrf_token();

    // Chargement depuis les données POST
    let mut form = RegisterForm::build_with_data(&data, ctx.engine.tera.clone(), &csrf).await;

    if form.is_valid().await {
        form.save(ctx.db()).await.unwrap();
        return Redirect::to("/success").into_response();
    }

    let mut context = ctx.context.clone();
    context.insert("form", form.get_form());
    ctx.render("register.html", &context)
}
```
