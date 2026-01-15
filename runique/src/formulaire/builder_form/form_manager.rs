use crate::formulaire::builder_form::trait_form::FormField;
use serde::ser::{SerializeStruct, Serializer};
use serde::{Serialize};
use serde_json::Value;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

pub struct Forms {
    pub fields: IndexMap<String, Box<dyn FormField>>,
    pub tera: Option<Arc<Tera>>,
    pub global_errors: Vec<String>,
}

impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
    {
        // On expose 5 champs pour Tera (is_valid est très utile en template)
        let mut state = serializer.serialize_struct("Forms", 5)?;
        state.serialize_field("cleaned_data", &self.cleaned_data())?;
        state.serialize_field("errors", &self.all_errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;
        state.serialize_field("html", &self.render_all().unwrap_or_default())?;
        state.serialize_field("is_valid", &self.is_valid_const())?;
        state.end()
    }
}

impl Forms {
    pub fn new() -> Self {
        Self {
            fields: IndexMap::new(),
            tera: None,
            global_errors: Vec::new(),
        }
    }

    // Ajout indispensable pour l'extracteur et la gestion Tera
    pub fn set_tera(&mut self, tera: Arc<Tera>) {
        self.tera = Some(tera);
    }

    pub fn add<T: FormField + 'static>(mut self, field: T) -> Self {
        self.fields.insert(field.name().to_string(), Box::new(field));
        self
    }

    pub fn fill(&mut self, data: &HashMap<String, String>) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }

    pub fn is_valid(&mut self) -> bool {
        let mut valid = true;
        for field in self.fields.values_mut() {
            if !field.validate() { valid = false; }
        }
        valid && self.global_errors.is_empty()
    }

    // Version "const" pour la sérialisation (ne modifie pas l'état)
    fn is_valid_const(&self) -> bool {
        self.global_errors.is_empty() &&
        self.fields.values().all(|f| f.get_error().is_none())
    }

    pub fn cleaned_data(&self) -> HashMap<String, Value> {
        self.fields.iter()
            .map(|(name, field)| (name.clone(), field.get_json_value()))
            .collect()
    }

    pub fn all_errors(&self) -> HashMap<String, String> {
        let mut errs: HashMap<String, String> = self.fields.iter()
            .filter_map(|(name, field)| {
                field.get_error().map(|err| (name.clone(), err.clone()))
            })
            .collect();

        if !self.global_errors.is_empty() {
            errs.insert("global".to_string(), self.global_errors.join(" | "));
        }
        errs
    }

    pub fn render_all(&self) -> Result<String, String> {
        let mut html = Vec::new();
        for field in self.fields.values() {
            html.push(field.render()?);
        }
        Ok(html.join("\n"))
    }
}