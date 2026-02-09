// --- Definitions communes pour les champs de formulaire ---
pub use crate::forms::base::FormField;
use crate::forms::manager::{Forms, ValidationError};
use async_trait::async_trait;
use sea_orm::DbErr;

use crate::utils::aliases::{ATera, StrMap};

dyn_clone::clone_trait_object!(FormField);

// Extrait des modifications à faire dans trait_form.rs

#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    async fn clean(&mut self) -> Result<(), StrMap> {
        Ok(())
    }

    async fn is_valid(&mut self) -> bool {
        // 1. Validation individuelle des champs (incluant CSRF)
        // Mot de passe est encore a cette étape.
        let fields_valid = match self.get_form_mut().is_valid() {
            Ok(valid) => valid,
            Err(ValidationError::StackOverflow) => {
                self.get_form_mut().global_errors.push(
                    "Stack overflow détecté : récursion infinie dans la validation".to_string(),
                );
                return false;
            }
            Err(_) => return false,
        };

        if !fields_valid {
            return false;
        }

        // 2. Validation métier croisée
        match self.clean().await {
            Ok(_) => {
                // 3. FINALISATION
                // Si tout est valide, on transforme les données (ex: Argon2)
                // et autre validation d'un dev
                if let Err(e) = self.get_form_mut().finalize() {
                    self.get_form_mut().global_errors.push(e);
                    return false;
                }
                true
            }
            Err(business_errors) => {
                let form = self.get_form_mut();
                for (name, msg) in business_errors {
                    if let Some(field) = form.fields.get_mut(&name) {
                        field.set_error(msg);
                    } else {
                        form.global_errors.push(msg);
                    }
                }
                false
            }
        }
    }

    fn database_error(&mut self, err: &DbErr) {
        self.get_form_mut().database_error(err);
    }

    fn build(tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);
        form.set_tera(tera);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    async fn build_with_data(raw_data: &StrMap, tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);

        form.set_tera(tera.clone());

        Self::register_fields(&mut form);

        form.fill(raw_data);

        let mut instance = Self::from_form(form);

        let _is_valid = instance.is_valid().await;

        instance
    }
}
