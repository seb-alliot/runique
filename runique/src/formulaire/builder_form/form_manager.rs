use crate::formulaire::builder_form::trait_form::FormField;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::Value;
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
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 5)?;
        state.serialize_field("cleaned_data", &self.cleaned_data())?;
        state.serialize_field("errors", &self.all_errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;

        let rendered_html = match self.render_all() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Erreur de rendu : {}</p>", e),
        };
        state.serialize_field("html", &rendered_html)?;

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
        let mut valid = true;
        for field in self.fields.values_mut() {
            if !field.validate() {
                valid = false;
            }
        }
        valid && self.global_errors.is_empty()
    }

    fn is_valid_const(&self) -> bool {
        self.global_errors.is_empty() && self.fields.values().all(|f| f.get_error().is_none())
    }

    pub fn cleaned_data(&self) -> HashMap<String, Value> {
        self.fields
            .iter()
            .map(|(name, field)| (name.clone(), field.get_json_value()))
            .collect()
    }

    pub fn all_errors(&self) -> HashMap<String, String> {
        let mut errs: HashMap<String, String> = self
            .fields
            .iter()
            .filter_map(|(name, field)| field.get_error().map(|err| (name.clone(), err.clone())))
            .collect();

        if !self.global_errors.is_empty() {
            errs.insert("global".to_string(), self.global_errors.join(" | "));
        }
        errs
    }

    pub fn render_all(&self) -> Result<String, String> {
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
    pub fn register_field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + 'static,
    {
        let field_instance = field_template.clone();
        self.add(field_instance);
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
                // On cherche le champ dans notre HashMap de champs
                if let Some(field) = self.fields.get_mut(&field_name) {
                    let friendly_name = field_name.replace("_", " ");
                    field.set_error(format!("Ce {} est déjà utilisé.", friendly_name));
                } else {
                    // Si on ne trouve pas le champ précis, on met une erreur globale
                    // Tu peux avoir un champ invisible "form_error" ou logger l'erreur
                    println!("Champ {} non trouvé pour l'erreur unique", field_name);
                }
            }
        } else {
            // Erreur générique si ce n'est pas un problème d'unicité
            // On peut l'attribuer au premier champ ou gérer une erreur globale
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

        // PostgreSQL anglais: unique constraint "users_username_key"
        if let Some(start) = err_msg.find("unique constraint \"") {
            let remaining = &err_msg[start + 19..];
            if let Some(end) = remaining.find('"') {
                let constraint_name = &remaining[..end];
                if let Some(parts) = Self::parse_constraint_name(constraint_name) {
                    return Some(parts);
                }
            }
        }
        // PostgreSQL Key (username)=(value)
        if let Some(start) = err_msg.find("Key (") {
            if let Some(end) = err_msg[start..].find(')') {
                let field = &err_msg[start + 5..start + end];
                return Some(field.to_string());
            }
        }

        // SQLite: UNIQUE constraint failed: users.username
        if let Some(pos) = err_msg.find("failed: ") {
            let remaining = &err_msg[pos + 8..];
            if let Some(dot_pos) = remaining.find('.') {
                let field = &remaining[dot_pos + 1..];
                let field_clean = field.split_whitespace().next()?;
                return Some(field_clean.to_string());
            }
        }

        // MySQL: Duplicate entry 'value' for key 'users.username'
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
        // Format typique: table_field_key ou table_field_idx
        let parts: Vec<&str> = constraint.split('_').collect();

        if parts.len() >= 3 {
            // Enlever le premier élément (nom de table) et le dernier (key/idx)
            let field_parts = &parts[1..parts.len() - 1];
            return Some(field_parts.join("_"));
        }

        None
    }
}
