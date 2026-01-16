// --- Definitions communes pour les champs de formulaire ---
use crate::formulaire::builder_form::form_manager::Forms;
use sea_orm::DbErr;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

use serde_json::Value;

pub trait FormField: Send + Sync + dyn_clone::DynClone {
    fn name(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn placeholder(&self) -> &str;

    fn set_name(&mut self, name: &str);
    fn set_label(&mut self, label: &str);
    // ----------------------------------------

    fn is_required(&self) -> bool {
        false
    }
    fn validate(&mut self) -> bool;
    fn get_error(&self) -> Option<&String>;
    fn set_error(&mut self, message: String);
    fn set_value(&mut self, value: &str);

    fn get_json_value(&self) -> Value {
        Value::String(self.value().to_string())
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String>;
}

dyn_clone::clone_trait_object!(FormField);

pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    fn build(tera: Arc<Tera>) -> Self {
        let mut form = Forms::new();
        form.set_tera(tera);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    fn build_with_current_data(raw_data: &HashMap<String, String>, tera: Arc<Tera>) -> Self {
        let mut form = Forms::new();
        form.set_tera(tera);
        Self::register_fields(&mut form);
        form.fill(raw_data);
        form.is_valid();
        Self::from_form(form)
    }
    fn database_error(&mut self, err: &DbErr) {
        self.get_form_mut().database_error(err);
    }
}
