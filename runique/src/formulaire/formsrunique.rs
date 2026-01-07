use crate::formulaire::field::RuniqueField;
use serde::de::DeserializeOwned;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Deserialize, Clone, Debug)]
pub struct Forms {
    pub errors: HashMap<String, String>,
    pub cleaned_data: HashMap<String, Value>,
    pub fields_html: Vec<(String, String)>,
}

// Implémenter Serialize manuellement pour inclure le HTML
impl Serialize for Forms {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Forms", 4)?;
        state.serialize_field("csrf_token", &self.fields_html)?;
        state.serialize_field("errors", &self.errors)?;
        state.serialize_field("cleaned_data", &self.cleaned_data)?;
        state.serialize_field("html", &self.render_html())?;
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
            fields_html: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.errors.clear();
        self.cleaned_data.clear();
        self.fields_html.clear();
    }

    /// Enregistrer un champ avec son HTML
    pub fn register_field<F: RuniqueField>(&mut self, name: &str, label: &str, field: &F) {
        let value_attr = self
            .cleaned_data
            .get(name)
            .map(|v| match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            })
            .unwrap_or_default();

        let mut html = field.render_html(name, label);

        if !value_attr.is_empty() {
            // 1. On cherche où commence la balise <input
            if let Some(input_start) = html.find("<input") {
                // 2. On cherche le '>' de CETTE balise input (après son début)
                if let Some(bracket_pos) = html[input_start..].find('>') {
                    let insert_pos: usize = input_start + bracket_pos;
                    let injection = format!(" value=\"{}\"", value_attr);
                    html.insert_str(insert_pos, &injection);
                }
            }
        }

        self.fields_html.push((name.to_string(), html));
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
                self.errors.insert(name.to_string(), "Requis".to_string());
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

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_not_valid(&self) -> bool {
        !self.is_valid()
    }

    pub fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(
        &self,
        field_name: &str,
    ) -> Option<T> {
        self.cleaned_data
            .get(field_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// Générer le HTML complet du formulaire
    pub fn render_html(&self) -> String {
        let mut html = String::new();
        for (field_name, field_html) in &self.fields_html {
            html.push_str(field_html);
            if let Some(error) = self.errors.get(field_name) {
                html.push_str(&format!(
                    r#"<div class="alert alert-danger">{}</div>"#,
                    error
                ));
            }
        }
        html
    }

    /// Générer le HTML d'un seul champ
    pub fn render_field(&self, field_name: &str) -> Option<String> {
        self.fields_html
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, html)| {
                let mut result = html.clone();

                // Ajouter l'erreur si présente
                if let Some(error) = self.errors.get(field_name) {
                    result.push_str(&format!(
                        r#"
    <div class="alert alert-danger">{}</div>"#,
                        error
                    ));
                }

                result
            })
    }
}

/// Trait pour l'auto-construction des formulaires
pub trait RuniqueForm: Sized {
    /// Enregistrer tous les champs du formulaire
    fn register_fields(form: &mut Forms);

    /// Valider tous les champs avec les données brutes
    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>);

    /// Construire le formulaire vide
    fn build() -> Self {
        let mut form = Forms::new();
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    /// Construire le formulaire avec validation (POST request)
    fn build_with_current_data(raw_data: &HashMap<String, String>) -> Self {
        let mut form = Forms::new();
        Self::validate_fields(&mut form, raw_data);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }
    /// Reconstruire le formulaire en excluant les champs sensibles
    fn rebuild_form(&self) -> Self {
        let mut data_map = HashMap::new();
        let sensitive_fields = ["password", "password_confirm", "current_password"];

        for (key, value) in self.get_cleaned_data() {
            if sensitive_fields.contains(&key.as_str()) {
                continue;
            }
            let val_str = match value {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                Value::Bool(b) => b.to_string(),
                _ => "".to_string(),
            };
            data_map.insert(key.clone(), val_str);
        }

        let mut new_form = Self::build_with_current_data(&data_map);

        for (field, error) in self.get_errors() {
            new_form
                .get_form_mut()
                .errors
                .insert(field.clone(), error.clone());
        }

        let internal = new_form.get_form_mut();
        internal.fields_html.clear();
        Self::register_fields(internal);
        for field in sensitive_fields {
            new_form.get_form_mut().errors.remove(field);
        }

        new_form
    }
    fn from_form(form: Forms) -> Self;

    /// Accéder au Forms interne (immutable)
    fn get_form(&self) -> &Forms;

    /// Accéder au Forms interne (mutable)
    fn get_form_mut(&mut self) -> &mut Forms;

    /// Générer le HTML complet du formulaire
    fn render_html(&self) -> String {
        self.get_form().render_html()
    }

    /// Vérifier si le formulaire est valide
    fn is_valid(&self) -> bool {
        self.get_form().is_valid()
    }

    /// Vérifier si le formulaire est invalide
    fn is_not_valid(&self) -> bool {
        self.get_form().is_not_valid()
    }

    /// Récupérer une valeur validée
    fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(
        &self,
        field_name: &str,
    ) -> Option<T> {
        self.get_form().get_value(field_name)
    }

    /// Récupérer toutes les erreurs
    fn get_errors(&self) -> &HashMap<String, String> {
        &self.get_form().errors
    }

    /// Récupérer toutes les données nettoyées
    fn get_cleaned_data(&self) -> &HashMap<String, Value> {
        &self.get_form().cleaned_data
    }
}

/// Trait original
pub trait FormulaireTrait: Send + Sync + 'static {
    fn new() -> Self;
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}
