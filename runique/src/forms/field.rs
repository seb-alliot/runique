// --- Common definitions for form fields ---
pub use crate::forms::base::FormField;
use crate::forms::form::Forms;
use crate::forms::renderer::FormRenderer; // ← Ajout
use crate::forms::validator::ValidationError;
use async_trait::async_trait;
use sea_orm::DbErr;

use crate::utils::aliases::{ATera, StrMap};

dyn_clone::clone_trait_object!(FormField);

#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    async fn clean(&mut self) -> Result<(), StrMap> {
        Ok(()) // Business validation hook
    }

    async fn is_valid(&mut self) -> bool {
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

        match self.clean().await {
            Ok(_) => {
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

    // ← Modifié : création du renderer ici
    fn build(tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);
        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    // ← Modifié : création du renderer ici aussi
    async fn build_with_data(raw_data: &StrMap, tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);

        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);

        Self::register_fields(&mut form);
        form.fill(raw_data);

        let mut instance = Self::from_form(form);
        let _is_valid = instance.is_valid().await;

        instance
    }
}
