// --- Common definitions for form fields ---
pub use crate::forms::base::FormField;
use crate::forms::form::Forms;
use crate::forms::renderer::FormRenderer;
use crate::forms::validator::ValidationError;
use crate::utils::aliases::{ATera, StrMap};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

dyn_clone::clone_trait_object!(FormField);

#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    // Business validation hook for individual fields
    async fn clean_field(&mut self, name: &str) -> bool {
        self.get_form().fields.contains_key(name)
    }

    // Business validation hook for the entire form
    async fn clean(&mut self) -> Result<(), StrMap> {
        Ok(())
    }

    async fn is_valid(&mut self) -> bool {
        let mut fields_valid = match self.get_form_mut().is_valid() {
            Ok(valid) => valid,
            Err(ValidationError::StackOverflow) => {
                self.get_form_mut().global_errors.push(
                    "Stack overflow détecté : récursion infinie dans la validation".to_string(),
                );
                return false;
            }
            Err(_) => return false,
        };

        let names: Vec<String> = self.get_form().fields.keys().cloned().collect();
        for name in names {
            if !self.clean_field(&name).await {
                fields_valid = false;
            }
        }

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

    /// Wrapper atomic par défaut : ouvre une transaction et appelle `save_txn`.
    ///
    /// - si `save_txn` renvoie Err -> rollback automatique
    /// - sinon -> commit
    async fn save_txn(&mut self, _txn: &DatabaseTransaction) -> Result<(), DbErr> {
        Ok(())
    }

    /// Wrapper atomic : transaction explicite (évite le piège des futures 'static)
    async fn save(&mut self, db: &DatabaseConnection) -> Result<(), DbErr> {
        let txn = db.begin().await?;

        match self.save_txn(&txn).await {
            Ok(()) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                // On tente rollback, mais on renvoie l’erreur métier/DB d’origine.
                let _ = txn.rollback().await;
                Err(e)
            }
        }
    }

    fn database_error(&mut self, err: &DbErr) {
        self.get_form_mut().database_error(err);
    }

    fn build(tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token);
        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

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
