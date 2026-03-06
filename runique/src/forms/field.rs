// --- Common definitions for form fields ---
pub use crate::forms::base::FormField;
use crate::forms::form::Forms;
use crate::forms::renderer::FormRenderer;
use crate::forms::validator::ValidationError;
use crate::utils::aliases::{ATera, StrMap};
use crate::utils::trad::t;
use async_trait::async_trait;
use axum::http::Method;
use sea_orm::{DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait};

dyn_clone::clone_trait_object!(FormField);

/// Trait principal pour les formulaires typés avec validation et sauvegarde
///
#[doc = include_str!("../../doc-tests/form/form_proc_macro.md")]
///
#[doc = include_str!("../../doc-tests/form/form_clean_save.md")]
#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    // Raccourcis directs — délèguent à get_form() pour éviter form.get_form().xxx()
    fn is_submitted(&self) -> bool {
        self.get_form().is_submitted()
    }
    fn get_value(&self, name: &str) -> Option<String> {
        self.get_form().get_value(name)
    }
    fn get_string(&self, name: &str) -> String {
        self.get_form().get_string(name)
    }
    fn get_option(&self, name: &str) -> Option<String> {
        self.get_form().get_option(name)
    }

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
                self.get_form_mut().errors.push(
                    t("forms.validation_overflow").into_owned(),
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
                    self.get_form_mut().errors.push(e);
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
                        form.errors.push(msg);
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

    async fn build_with_data(
        raw_data: &StrMap,
        tera: ATera,
        csrf_token: &str,
        method: Method,
    ) -> Self {
        let mut form = Forms::new(csrf_token);

        let renderer = FormRenderer::new(tera);
        form.set_renderer(renderer);

        Self::register_fields(&mut form);
        form.fill(raw_data, method);
        Self::from_form(form)
    }
}
