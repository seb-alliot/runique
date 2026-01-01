use crate::formulaire::field::RustiField;
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
        let mut state = serializer.serialize_struct("Forms", 3)?;
        state.serialize_field("errors", &self.errors)?;
        state.serialize_field("cleaned_data", &self.cleaned_data)?;
        state.serialize_field("html", &self.render_html())?;
        state.end()
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
    pub fn register_field<F: RustiField>(&mut self, name: &str, label: &str, field: &F) {
        let html = field.render_html(name, label);
        self.fields_html.push((name.to_string(), html));
    }

    pub fn require<F: RustiField>(
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

    pub fn optional<F: RustiField>(
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

    pub fn field<F: RustiField>(
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
        // Boucle sur tous les champs enregistrés
        for (field_name, field_html) in &self.fields_html {
            html.push_str(&field_html);

            // Ajouter les erreurs si présentes
            if let Some(error) = self.errors.get(field_name) {
                html.push_str(&format!(
                    r#"
    <div class="alert alert-danger">{}</div>"#,
                    error
                ));
            }
        }

        html
    }

    /// Générer le HTML d'un seul champ (pour templates)
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
pub trait RustiForm: Sized {
    /// Enregistrer tous les champs du formulaire
    fn register_fields(form: &mut Forms);

    /// Valider tous les champs avec les données brutes
    fn validate_fields(form: &mut Forms, raw_data: &HashMap<String, String>);

    /// Construire le formulaire vide (GET request)
    fn build() -> Self {
        let mut form = Forms::new();
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    /// Construire le formulaire avec validation (POST request)
    fn build_with_data(raw_data: &HashMap<String, String>) -> Self {
        let mut form = Forms::new();
        Self::register_fields(&mut form);
        Self::validate_fields(&mut form, raw_data);
        Self::from_form(form)
    }

    /// Créer l'instance depuis Forms
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

/// Trait original (pour compatibilité)
pub trait FormulaireTrait: Send + Sync + 'static {
    fn new() -> Self;
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}
