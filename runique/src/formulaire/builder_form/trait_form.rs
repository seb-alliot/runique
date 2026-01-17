// --- Definitions communes pour les champs de formulaire ---
use crate::formulaire::builder_form::form_manager::Forms;
use sea_orm::DbErr;
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tera::Tera;

pub trait FormField: Send + Sync + dyn_clone::DynClone {
    // Getters
    fn name(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn placeholder(&self) -> &str;
    fn field_type(&self) -> &str;
    fn error(&self) -> Option<&String>;

    fn is_required(&self) -> bool {
        false
    }

    // Setters
    fn set_name(&mut self, name: &str);
    fn set_label(&mut self, label: &str);
    fn set_placeholder(&mut self, placeholder: &str);
    fn set_value(&mut self, value: &str);
    fn set_error(&mut self, message: String);
    fn set_readonly(&mut self, _readonly: bool, _msg: Option<&str>) {}
    fn set_disabled(&mut self, _disabled: bool, _msg: Option<&str>) {}
    fn set_required(&mut self, required: bool, msg: Option<&str>);
    fn set_html_attribute(&mut self, key: &str, value: &str);

    // Validation
    fn validate(&mut self) -> bool;

    // Rendu
    fn render(&self, tera: &Arc<Tera>) -> Result<String, String>;

    // Conversion JSON
    fn to_json_value(&self) -> Value {
        Value::String(self.value().to_string())
    }

    fn to_json_required(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": self.is_required(),
            "message": null
        })
    }

    fn to_json_readonly(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": false,
            "message": null
        })
    }

    fn to_json_disabled(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": false,
            "message": null
        })
    }

    fn to_json_attributes(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

dyn_clone::clone_trait_object!(FormField);

pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    fn clean(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), HashMap<String, String>>> + Send + '_>> {
        Box::pin(async move { Ok(()) })
    }

    fn is_valid(&mut self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move {
            let fields_valid = self.get_form_mut().is_valid().await;

            if !fields_valid {
                return false;
            }

            match self.clean().await {
                Ok(_) => true,
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
        })
    }

    fn build(tera: Arc<Tera>) -> Self {
        let mut form = Forms::new();
        form.set_tera(tera);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    fn build_with_data(raw_data: &HashMap<String, String>, tera: Arc<Tera>) -> Self {
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
