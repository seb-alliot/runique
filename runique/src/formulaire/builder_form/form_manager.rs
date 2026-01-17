use crate::formulaire::builder_form::generique_field::GenericField;
use crate::formulaire::builder_form::trait_form::FormField;
use crate::sea_orm;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;
#[derive(Clone)]
pub struct Forms {
    /// On stocke des Box de FormField.
    /// En pratique, tout sera converti en GenericField pour la cohérence.
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

        let fields_data: HashMap<String, serde_json::Value> = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, (name, field))| {
                let mut field_map = serde_json::Map::new();
                field_map.insert("name".to_string(), json!(name));
                field_map.insert("label".to_string(), json!(field.label()));
                field_map.insert("field_type".to_string(), json!(field.field_type()));
                field_map.insert("value".to_string(), json!(field.value()));
                field_map.insert("placeholder".to_string(), json!(field.placeholder()));
                field_map.insert("index".to_string(), json!(index));

                if let Some(err) = field.error() {
                    field_map.insert("error".to_string(), json!(err));
                }

                field_map.insert("is_required".to_string(), field.to_json_required());
                field_map.insert("readonly".to_string(), field.to_json_readonly());
                field_map.insert("disabled".to_string(), field.to_json_disabled());
                field_map.insert("html_attributes".to_string(), field.to_json_attributes());

                (name.clone(), Value::Object(field_map))
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

    /// La solution au "type annotations needed" :
    /// On force la conversion en GenericField ici même.
    pub fn field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + Into<GenericField> + 'static,
    {
        let generic_instance: GenericField = field_template.clone().into();
        self.add(generic_instance);
    }

    pub fn add<T: FormField + 'static>(&mut self, field: T) -> &mut Self {
        let name = field.name().to_string();
        self.fields.insert(name, Box::new(field));
        self
    }

    pub fn set_tera(&mut self, tera: Arc<Tera>) {
        self.tera = Some(tera);
    }

    pub fn fill(&mut self, data: &HashMap<String, String>) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }

    pub async fn is_valid(&mut self) -> bool {
        let mut is_all_valid = true;
        for field in self.fields.values_mut() {
            if field.is_required() && field.value().trim().is_empty() {
                field.set_error("Ce champ est obligatoire".to_string());
                is_all_valid = false;
                continue;
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
        let tera_instance = self.tera.as_ref().ok_or("Tera non configuré")?;

        for field in self.fields.values() {
            match field.render(tera_instance) {
                Ok(rendered) => html.push(rendered),
                Err(e) => return Err(format!("Erreur rendu '{}': {}", field.name(), e)),
            }
        }
        Ok(html.join("\n"))
    }

    pub fn get_value(&self, name: &str) -> Option<String> {
        self.fields.get(name).map(|field| field.value().to_string())
    }

    pub fn database_error(&mut self, db_err: &sea_orm::DbErr) {
        let err_msg = db_err.to_string();
        // Logique simplifiée d'extraction (à enrichir selon les besoins)
        if err_msg.contains("unique") || err_msg.contains("Duplicate") {
            self.global_errors
                .push("Une contrainte d'unicité a été violée.".to_string());
        } else {
            self.global_errors.push(format!("Erreur DB: {}", err_msg));
        }
    }
}
