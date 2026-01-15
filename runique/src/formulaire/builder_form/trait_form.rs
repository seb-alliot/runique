// --- Definitions communes pour les champs de formulaire ---
use std::collections::HashMap;
use std::sync::Arc;
use crate::formulaire::builder_form::form_manager::Forms;
use tera::Tera;

use serde_json::Value;

pub trait FormField: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn placeholder(&self) -> &str;

    // Par défaut, un champ n'est pas requis
    fn is_required(&self) -> bool { false }

    fn validate(&mut self) -> bool;
    fn get_error(&self) -> Option<&String>;
    fn set_error(&mut self, message: String);
    fn set_value(&mut self, value: &str);

    fn get_json_value(&self) -> Value {
        Value::String(self.value().to_string())
    }

    fn render(&self) -> Result<String, String>;
}


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
}