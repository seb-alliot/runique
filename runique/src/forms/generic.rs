use crate::forms::field::FormField;
use crate::forms::fields::boolean::BooleanField;
use crate::forms::fields::file::FileField;
use crate::forms::fields::{NumericField, TextField};
use serde::Serialize;
use serde_json::Value;

use crate::utils::aliases::ATera;

#[derive(Clone, Debug, Serialize)]
pub enum FieldKind {
    Text(TextField),
    Numeric(NumericField),
    Boolean(BooleanField),
    #[serde(skip)]
    File(FileField),
}

#[derive(Clone, Serialize, Debug)]
pub struct GenericField {
    pub kind: FieldKind,
}

impl FormField for GenericField {
    fn name(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.name(),
            FieldKind::Numeric(f) => f.name(),
            FieldKind::File(f) => f.name(),
            _ => "",
        }
    }

    fn label(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.label(),
            FieldKind::Numeric(f) => f.label(),
            FieldKind::File(f) => f.label(),
            _ => "",
        }
    }

    fn value(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.value(),
            FieldKind::Numeric(f) => f.value(),
            FieldKind::File(f) => f.value(),
            _ => "",
        }
    }

    fn placeholder(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.placeholder(),
            FieldKind::Numeric(f) => f.placeholder(),
            FieldKind::File(f) => f.placeholder(),
            _ => "",
        }
    }

    fn required(&self) -> bool {
        match &self.kind {
            FieldKind::Text(f) => f.required(),
            FieldKind::Numeric(f) => f.required(),
            FieldKind::File(f) => f.required(),
            _ => false,
        }
    }

    fn error(&self) -> Option<&String> {
        match &self.kind {
            FieldKind::Text(f) => f.error(),
            FieldKind::Numeric(f) => f.error(),
            FieldKind::File(f) => f.error(),
            _ => None,
        }
    }

    // --- Délégation des modificateurs (Setters) ---

    fn set_value(&mut self, value: &str) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_value(value),
            FieldKind::Numeric(f) => f.set_value(value),
            FieldKind::File(f) => f.set_value(value),
            _ => {}
        }
    }

    fn set_error(&mut self, error: String) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_error(error),
            FieldKind::Numeric(f) => f.set_error(error),
            FieldKind::File(f) => f.set_error(error),
            _ => {}
        }
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_placeholder(placeholder),
            FieldKind::Numeric(f) => f.set_placeholder(placeholder),
            FieldKind::File(f) => f.set_placeholder(placeholder),
            _ => {}
        }
    }

    // --- Logique métier ---

    fn validate(&mut self) -> bool {
        match &mut self.kind {
            FieldKind::Text(f) => f.validate(),
            FieldKind::Numeric(f) => f.validate(),
            FieldKind::File(f) => f.validate(),
            _ => true,
        }
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        match &self.kind {
            FieldKind::Text(f) => f.render(tera),
            FieldKind::Numeric(f) => f.render(tera),
            FieldKind::File(f) => f.render(tera),
            _ => Err("Type de champ non supporté pour le rendu".into()),
        }
    }
    fn set_name(&mut self, name: &str) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_name(name),
            FieldKind::Numeric(f) => f.set_name(name),
            FieldKind::File(f) => f.set_name(name),
            _ => {}
        }
    }
    fn set_html_attribute(&mut self, key: &str, value: &str) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_html_attribute(key, value),
            FieldKind::Numeric(f) => f.set_html_attribute(key, value),
            FieldKind::File(f) => f.set_html_attribute(key, value),
            _ => {}
        }
    }
    fn field_type(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.field_type(),
            FieldKind::Numeric(f) => f.field_type(),
            FieldKind::File(f) => f.field_type(),
            _ => "",
        }
    }
    fn template_name(&self) -> &str {
        match &self.kind {
            FieldKind::Text(f) => f.template_name(),
            FieldKind::Numeric(f) => f.template_name(),
            FieldKind::Boolean(f) => f.template_name(),
            FieldKind::File(f) => f.template_name(),
        }
    }
    // --- Sérialisation JSON ---

    fn to_json_value(&self) -> Value {
        match &self.kind {
            FieldKind::Text(f) => f.to_json_value(),
            FieldKind::Numeric(f) => f.to_json_value(),
            FieldKind::File(f) => f.to_json_value(),
            _ => serde_json::json!(null),
        }
    }
    fn to_json_required(&self) -> Value {
        match &self.kind {
            FieldKind::Text(f) => f.to_json_required(),
            FieldKind::Numeric(f) => f.to_json_required(),
            FieldKind::File(f) => f.to_json_required(),
            _ => serde_json::json!(false),
        }
    }
    fn to_json_attributes(&self) -> Value {
        match &self.kind {
            FieldKind::Text(f) => f.to_json_attributes(),
            FieldKind::Numeric(f) => f.to_json_attributes(),
            FieldKind::File(f) => f.to_json_attributes(),
            _ => serde_json::json!({}),
        }
    }

    fn set_readonly(&mut self, _readonly: bool, _msg: Option<&str>) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_readonly(_readonly, _msg),
            FieldKind::Numeric(f) => f.set_readonly(_readonly, _msg),
            FieldKind::File(f) => f.set_readonly(_readonly, _msg),
            _ => {}
        }
    }
    fn set_disabled(&mut self, _disabled: bool, _msg: Option<&str>) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_disabled(_disabled, _msg),
            FieldKind::Numeric(f) => f.set_disabled(_disabled, _msg),
            FieldKind::File(f) => f.set_disabled(_disabled, _msg),
            _ => {}
        }
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_required(required, msg),
            FieldKind::Numeric(f) => f.set_required(required, msg),
            FieldKind::File(f) => f.set_required(required, msg),
            _ => {}
        }
    }

    fn set_label(&mut self, label: &str) {
        match &mut self.kind {
            FieldKind::Text(f) => f.set_label(label),
            FieldKind::Numeric(f) => f.set_label(label),
            FieldKind::File(f) => f.set_label(label),
            _ => {}
        }
    }
    // Ajoute ici les autres méthodes du trait FormField (readonly, disabled, etc.)
    // en suivant toujours le même schéma : match &self.kind { ... }
}

// Helper pour faciliter l'emballage
impl From<TextField> for GenericField {
    fn from(f: TextField) -> Self {
        Self {
            kind: FieldKind::Text(f),
        }
    }
}

impl From<NumericField> for GenericField {
    fn from(f: NumericField) -> Self {
        Self {
            kind: FieldKind::Numeric(f),
        }
    }
}

impl From<FileField> for GenericField {
    fn from(f: FileField) -> Self {
        Self {
            kind: FieldKind::File(f),
        }
    }
}
