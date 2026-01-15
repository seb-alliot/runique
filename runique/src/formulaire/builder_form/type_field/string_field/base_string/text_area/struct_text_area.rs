use crate::formulaire::validation_form::builder_form::trait_form::FormField;
use crate::formulaire::validation_form::builder_form::base_struct::FieldConfig;

use crate::app::TERA;
use tera::Context;

pub struct TextAreaField {
    pub base : FieldConfig,
    pub max_length: Option<usize>,
    pub rows: u32,
}

impl TextAreaField {
    pub fn new(id: &str, name: &str, label: &str) -> Self {
        Self {
            base: FieldConfig {
                id: id.to_string(),
                name: name.to_string(),
                label: label.to_string(),
                value: String::new(),
                placeholder: String::new(),
                error: None,
                is_required: false,
            },
            max_length: None,
            rows: 3,
        }
    }

    // Builder pour le placeholder (il manquait celui-là !)
    pub fn placeholder(mut self, p: &str) -> Self {
        self.base.placeholder = p.to_string();
        self
    }

    pub fn rows(mut self, rows: u32) -> Self {
        self.rows = rows;
        self
    }

    pub fn required(mut self) -> Self {
        self.base.is_required = true;
        self
    }
}

impl FormField for TextAreaField {
    fn id(&self) -> &str { &self.base.id }
    fn name(&self) -> &str { &self.base.name }
    fn label(&self) -> &str { &self.base.label }
    fn value(&self) -> &str { &self.base.value }
    fn placeholder(&self) -> &str { &self.base.placeholder }
    fn get_error(&self) -> Option<&String> { self.base.error.as_ref() }

    fn validate(&mut self) -> bool {
        if self.base.is_required && self.base.value.trim().is_empty() {
            self.base.error = Some("Ce champ est obligatoire".to_string());
            return false;
        }
        if let Some(max) = self.max_length {
            if self.base.value.chars().count() > max {
                self.base.error = Some(format!("Maximum {} caractères", max));
                return false;
            }
        }
        self.base.error = None;
        true
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn render(&self) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("id", &self.base.id);
        context.insert("name", &self.base.name);
        context.insert("label", &self.base.label);
        context.insert("value", &self.base.value);
        context.insert("placeholder", &self.base.placeholder);
        context.insert("error", &self.base.error);
        context.insert("required", &self.base.is_required);
        context.insert("max_length", &self.max_length);
        context.insert("rows", &self.rows);

        TERA.render("text_area", &context)
            .map_err(|e| format!("Erreur rendu TextArea ({}): {}", self.base.id, e))
    }
}