use crate::formulaire::builder_form::trait_form::FormField;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

#[derive(Clone)]
pub struct Forms {
    pub fields: IndexMap<String, Box<dyn FormField>>,
    pub tera: Option<Arc<Tera>>,
    pub global_errors: Vec<String>,
}

impl Default for Forms {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Forms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forms")
            .field("fields_count", &self.fields.len())
            .field("has_tera", &self.tera.is_some())
            .field("global_errors", &self.global_errors)
            .finish()
    }
}

impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 5)?;
        state.serialize_field("data", &self.data())?;
        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Erreur de rendu : {}</p>", e),
        };
        state.serialize_field("html", &rendered_html)?;

        // Sérialiser les champs avec leurs métadonnées complètes
        let fields_data: HashMap<String, serde_json::Value> = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, (name, field))| {
                let mut field_map = serde_json::Map::new();
                field_map.insert("name".to_string(), serde_json::Value::String(name.clone()));
                field_map.insert(
                    "label".to_string(),
                    serde_json::Value::String(field.label().to_string()),
                );
                field_map.insert(
                    "field_type".to_string(),
                    serde_json::Value::String(field.field_type().to_string()),
                );
                field_map.insert(
                    "value".to_string(),
                    serde_json::Value::String(field.value().to_string()),
                );
                field_map.insert(
                    "placeholder".to_string(),
                    serde_json::Value::String(field.placeholder().to_string()),
                );
                field_map.insert("index".to_string(), serde_json::Value::Number(index.into()));

                if let Some(err) = field.error() {
                    field_map.insert("error".to_string(), serde_json::Value::String(err.clone()));
                }

                // Utiliser les nouvelles méthodes du trait
                field_map.insert("is_required".to_string(), field.to_json_required());
                field_map.insert("readonly".to_string(), field.to_json_readonly());
                field_map.insert("disabled".to_string(), field.to_json_disabled());
                field_map.insert("html_attributes".to_string(), field.to_json_attributes());

                (name.clone(), serde_json::Value::Object(field_map))
            })
            .collect();
        state.serialize_field("fields", &fields_data)?;

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

    pub fn field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + 'static,
    {
        let field_instance = field_template.clone();
        self.add(field_instance);
    }

    pub fn set_tera(&mut self, tera: Arc<Tera>) {
        self.tera = Some(tera);
    }

    pub fn add<T: FormField + 'static>(&mut self, field: T) -> &mut Self {
        let name = field.name().to_string();
        self.fields.insert(name, Box::new(field));
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
        let mut is_all_valid = true;
        for field in self.fields.values_mut() {
            if field.is_required() && field.value().is_empty() {
                field.set_error("Ce champ est obligatoire".to_string());
                is_all_valid = false;
            }
            if !field.validate() {
                is_all_valid = false;
            }
        }
        is_all_valid && self.global_errors.is_empty()
    }

    pub fn has_errors(&self) -> bool {
        !self.global_errors.is_empty() || self.fields.values().any(|f| f.error().is_some())
    }

    pub fn data(&self) -> HashMap<String, Value> {
        self.fields
            .iter()
            .map(|(name, field)| (name.clone(), field.to_json_value()))
            .collect()
    }

    pub fn errors(&self) -> HashMap<String, String> {
        let mut errs: HashMap<String, String> = self
            .fields
            .iter()
            .filter_map(|(name, field)| field.error().map(|err| (name.clone(), err.clone())))
            .collect();

        if !self.global_errors.is_empty() {
            errs.insert("global".to_string(), self.global_errors.join(" | "));
        }
        errs
    }

    pub fn render(&self) -> Result<String, String> {
        let mut html = Vec::new();
        let tera_instance = self.tera.as_ref().ok_or("Tera not set")?;

        for field in self.fields.values() {
            match field.render(tera_instance) {
                Ok(rendered) => html.push(rendered),
                Err(e) => {
                    return Err(format!("Erreur sur le champ '{}': {:?}", field.name(), e));
                }
            }
        }
        Ok(html.join("\n"))
    }

    pub fn get_value(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|field| field.value().to_string())
    }

    pub fn get_value_or_default(&self, name: &str) -> String {
        self.get_value(name).unwrap_or_default()
    }

    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        let err_msg = db_err.to_string();

        if err_msg.contains("unique") || err_msg.contains("UNIQUE") || err_msg.contains("Duplicate")
        {
            if let Some(field_name) = Self::extract_field_name(&err_msg) {
                if let Some(field) = self.fields.get_mut(&field_name) {
                    let friendly_name = field_name.replace("_", " ");
                    field.set_error(format!("Ce {} est déjà utilisé.", friendly_name));
                } else {
                    self.global_errors
                        .push("Une erreur de base de données est survenue.".to_string());
                }
            }
        } else {
            if let Some((_, first_field)) = self.fields.iter_mut().next() {
                first_field.set_error("Une erreur de base de données est survenue.".to_string());
            }
        }
    }

    fn extract_field_name(err_msg: &str) -> Option<String> {
        if let Some(start) = err_msg.find("contrainte unique « ") {
            let remaining = &err_msg[start + 20..];
            if let Some(end) = remaining.find(" »") {
                let constraint_name = &remaining[..end];
                if let Some(parts) = Self::parse_constraint_name(constraint_name) {
                    return Some(parts);
                }
            }
        }

        if let Some(start) = err_msg.find("unique constraint \"") {
            let remaining = &err_msg[start + 19..];
            if let Some(end) = remaining.find('"') {
                let constraint_name = &remaining[..end];
                if let Some(parts) = Self::parse_constraint_name(constraint_name) {
                    return Some(parts);
                }
            }
        }

        if let Some(start) = err_msg.find("Key (") {
            if let Some(end) = err_msg[start..].find(')') {
                let field = &err_msg[start + 5..start + end];
                return Some(field.to_string());
            }
        }

        if let Some(pos) = err_msg.find("failed: ") {
            let remaining = &err_msg[pos + 8..];
            if let Some(dot_pos) = remaining.find('.') {
                let field = &remaining[dot_pos + 1..];
                let field_clean = field.split_whitespace().next()?;
                return Some(field_clean.to_string());
            }
        }

        if let Some(pos) = err_msg.find("for key '") {
            let remaining = &err_msg[pos + 9..];
            if let Some(dot_pos) = remaining.find('.') {
                let after_dot = &remaining[dot_pos + 1..];
                if let Some(quote_pos) = after_dot.find('\'') {
                    return Some(after_dot[..quote_pos].to_string());
                }
            }
        }

        None
    }

    fn parse_constraint_name(constraint: &str) -> Option<String> {
        let parts: Vec<&str> = constraint.split('_').collect();

        if parts.len() >= 3 {
            let field_parts = &parts[1..parts.len() - 1];
            return Some(field_parts.join("_"));
        }

        None
    }
}
