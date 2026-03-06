# Validation métier : `clean_field`, `clean` et `save`

```rust,ignore
use runique::prelude::*;
use runique::forms::{RuniqueForm, Forms};
use runique::forms::fields::text::TextField;
use async_trait::async_trait;
use sea_orm::{DatabaseTransaction, DbErr, Set};
use std::collections::HashMap;

pub struct ChangePasswordForm {
    form: Forms,
}

#[async_trait]
impl RuniqueForm for ChangePasswordForm {
    impl_form_access!();

    fn register_fields(form: &mut Forms) {
        form.field(&TextField::password("password").label("Nouveau mot de passe").required(true, None));
        form.field(&TextField::password("confirm").label("Confirmation").required(true, None));
    }

    /// Validation par champ — appelée pour chaque champ après is_valid()
    async fn clean_field(&mut self, name: &str) -> bool {
        match name {
            "password" => {
                let pwd = self.get_form().get_string("password");
                if pwd.len() < 8 {
                    if let Some(field) = self.get_form_mut().fields.get_mut("password") {
                        field.set_error("Le mot de passe doit faire au moins 8 caractères".to_string());
                    }
                    return false;
                }
                true
            }
            _ => true,
        }
    }

    /// Validation globale — appelée après tous les clean_field()
    async fn clean(&mut self) -> Result<(), HashMap<String, String>> {
        let password = self.get_form().get_string("password");
        let confirm  = self.get_form().get_string("confirm");

        if password != confirm {
            let mut errors = HashMap::new();
            errors.insert("confirm".to_string(), "Les mots de passe ne correspondent pas".to_string());
            return Err(errors);
        }

        Ok(())
    }

    /// Sauvegarde dans une transaction — appelée par save()
    async fn save_txn(&mut self, txn: &DatabaseTransaction) -> Result<(), DbErr> {
        let new_password = self.get_form().get_string("password");
        let hashed = runique::utils::password::hash_password(&new_password);

        // Mise à jour en base...
        // UserActiveModel { password: Set(hashed), ..Default::default() }
        //     .update(txn).await?;

        Ok(())
    }
}

// Utilisation dans un handler
async fn change_password_handler(
    ctx: Request,
    Form(data): Form<HashMap<String, String>>,
    mut message: Message,
) -> Response {
    let csrf = ctx.csrf_token();
    let mut form = ChangePasswordForm::build_with_data(&data, ctx.engine.tera.clone(), &csrf).await;

    if form.is_valid().await {
        match form.save(ctx.db()).await {
            Ok(_) => {
                success!(message => "Mot de passe modifié avec succès");
                return Redirect::to("/profile").into_response();
            }
            Err(e) => {
                form.database_error(&e);
            }
        }
    }

    let mut context = ctx.context.clone();
    context.insert("form", form.get_form());
    ctx.render("change_password.html", &context)
}
```
