# Validation métier : `clean_field` et `clean`

```rust,ignore
use runique::prelude::*;
use runique::forms::{RuniqueForm, Forms};
use runique::forms::fields::text::TextField;

pub struct ChangePasswordForm {
    form: Forms,
}

impl FormAccess for ChangePasswordForm {
    impl_form_access!();
    fn register_fields(form: &mut Forms) {
        form.field(&TextField::password("password").label("Nouveau mot de passe").required());
        form.field(&TextField::password("confirm").label("Confirmation").required());
    }
}

#[async_trait]
impl RuniqueForm for ChangePasswordForm {
    /// Validation par champ — appelée pour chaque champ par is_valid()
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
    /// Validation globale inter-champs — appelée après tous les clean_field()
    async fn clean(&mut self) -> Result<(), StrMap> {
        let password = self.get_form().get_string("password");
        let confirm  = self.get_form().get_string("confirm");

        if password != confirm {
            let mut errors = StrMap::new();
            errors.insert("confirm".to_string(), "Les mots de passe ne correspondent pas".to_string());
            return Err(errors);
        }

        Ok(())
    }
}
```
