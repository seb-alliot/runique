use crate::formulaire::field::RuniqueField;
use serde::de::DeserializeOwned;
use serde::ser::{SerializeStruct, Serializer};
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tera::Tera;

#[derive(Clone, Deserialize, Debug)]
pub struct Forms {
    pub errors: HashMap<String, String>,
    pub cleaned_data: HashMap<String, Value>,
    pub fields_html: indexmap::IndexMap<String, String>,

    #[serde(skip)]
    pub tera: Option<Arc<Tera>>,
}

impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 4)?;
        state.serialize_field("errors", &self.errors)?;
        state.serialize_field("cleaned_data", &self.cleaned_data)?;
        state.serialize_field("html", &self.render_html())?;

        state.serialize_field("fields", &self.fields_html)?;

        state.end()
    }
}

impl Default for Forms {
    fn default() -> Self {
        Self::new()
    }
}

impl Forms {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            cleaned_data: HashMap::new(),
            fields_html: indexmap::IndexMap::new(),
            tera: None,
        }
    }

    /// Associer l'instance Tera au formulaire
    pub fn set_tera(&mut self, tera: Arc<Tera>) {
        self.tera = Some(tera);
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.cleaned_data.clear();
        self.fields_html.clear();
    }

    /// Enregistrer un champ et générer son HTML via Tera
    pub fn register_field<F: RuniqueField>(&mut self, name: &str, label: &str, field: &F) {
        if let Some(ref tera_instance) = self.tera {
            let value = self.cleaned_data.get(name).unwrap_or(&Value::Null);
            let error = self.errors.get(name);

            let html = field.render(tera_instance, name, label, value, error);

            self.fields_html.insert(name.to_string(), html);
        } else {
            self.fields_html.insert(
                name.to_string(),
                "<p>Erreur : Tera non initialisé</p>".to_string(),
            );
        }
    }

    pub fn field<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_value: &str,
    ) -> Option<F::Output>
    where
        F::Output: Serialize + Clone,
    {
        let value_to_process = if field.strip() {
            raw_value.trim()
        } else {
            raw_value
        };

        match field.process(value_to_process) {
            Ok(value) => {
                if let Ok(json_val) = serde_json::to_value(value.clone()) {
                    self.cleaned_data.insert(name.to_string(), json_val);
                }
                Some(value)
            }
            Err(e) => {
                self.errors.insert(name.to_string(), e);
                None
            }
        }
    }

    pub fn require<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_data: &HashMap<String, String>,
    ) where
        F::Output: Serialize + Clone,
    {
        match raw_data.get(name) {
            Some(value) => {
                self.field(name, field, value);
            }
            None => {
                self.errors
                    .insert(name.to_string(), "Ce champ est requis".to_string());
            }
        }
    }

    pub fn optional<F: RuniqueField>(
        &mut self,
        name: &str,
        field: &F,
        raw_data: &HashMap<String, String>,
    ) where
        F::Output: Serialize + Clone,
    {
        if let Some(value) = raw_data.get(name) {
            self.field(name, field, value);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
    pub fn is_not_valid(&self) -> bool {
        !self.is_valid()
    }

    pub fn render_html(&self) -> String {
        self.fields_html
            .values()
            .cloned()
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(
        &self,
        field_name: &str,
    ) -> Option<T> {
        self.cleaned_data
            .get(field_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }
}

pub trait RuniqueForm: Sized {
    fn register_fields(form: &mut Forms);
    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>);

    /// Pour un formulaire vide : on génère le HTML
    fn build(tera: Arc<Tera>) -> Self {
        let mut form = Forms::new();
        form.set_tera(tera);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    /// Pour une soumission : on valide, PUIS on génère le HTML (une seule fois)
    fn build_with_current_data(raw_data: &HashMap<String, String>, tera: Arc<Tera>) -> Self {
        let mut form = Forms::new();
        form.set_tera(tera);

        // 1. On traite les DATA d'abord (sans toucher au HTML)
        Self::validate_fields(&mut form, raw_data);

        // 2. On génère le HTML à la toute fin, une seule fois.
        // L'IndexMap se remplira dans l'ordre exact défini ici.
        Self::register_fields(&mut form);

        Self::from_form(form)
    }

    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    // Proxies pratiques
    fn render_html(&self) -> String {
        self.get_form().render_html()
    }
    fn is_valid(&self) -> bool {
        self.get_form().is_valid()
    }
    fn get_cleaned_data(&self) -> &HashMap<String, Value> {
        &self.get_form().cleaned_data
    }
    fn get_errors(&self) -> &HashMap<String, String> {
        &self.get_form().errors
    }
}
