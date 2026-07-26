//! `FormField` trait and `FieldConfig` structure: common base for all form fields.
use crate::forms::options::*;
use crate::utils::aliases::*;
use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

/// Common configuration for a form field (name, label, value, error, HTML attributes).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_required: BoolChoice,
    pub error: Option<String>,
    pub type_field: String,
    pub html_attributes: StrMap,
    pub template_name: String,
    pub extra_context: JsonMap,
    /// Champ dont la valeur ne doit **jamais** ressortir : ni dans le widget rendu,
    /// ni dans un audit, ni dans une vue d'administration.
    ///
    /// Privé et sans setter d'extinction : la seule façon de l'activer est
    /// [`FieldConfig::mark_password`], il n'existe aucun moyen de le désactiver.
    /// Se lit par [`FieldConfig::is_password`], qui vaut également `true` pour tout
    /// champ de type `password` — un champ sensible reste donc protégé même si
    /// personne n'a pensé à poser le drapeau.
    #[serde(default)]
    is_password: bool,
}

impl FieldConfig {
    pub fn new(name: &str, type_field: &str, template_name: &str) -> Self {
        Self {
            name: name.to_string(),
            label: String::new(),
            value: String::new(),
            placeholder: String::new(),
            is_required: BoolChoice::default(),
            error: None,
            type_field: type_field.to_string(),
            html_attributes: HashMap::new(),
            template_name: template_name.to_string(),
            extra_context: HashMap::new(),
            // Dérivé du type dès la construction : tout champ `password`, quel que
            // soit le constructeur emprunté, naît protégé. Le drapeau ne peut donc
            // pas être manqué par omission.
            is_password: type_field == "password",
        }
    }

    /// `true` si la valeur de ce champ ne doit jamais être exposée.
    ///
    /// Le type l'emporte sur le drapeau : même si le champ a été construit à la
    /// main sans passer par `mark_password`, un `type_field` valant `password`
    /// suffit à le protéger.
    #[must_use]
    pub fn is_password(&self) -> bool {
        self.is_password || self.type_field == "password"
    }

    /// Marque le champ comme portant un secret (clé d'API, jeton…).
    ///
    /// Sens unique, volontairement : il n'existe pas d'opération inverse. Un champ
    /// déclaré sensible ne peut pas cesser de l'être en cours de route, ce qui
    /// interdit qu'un chemin de code le « démasque » par erreur.
    pub fn mark_password(&mut self) {
        self.is_password = true;
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<LengthConstraint>,
    pub min_length: Option<LengthConstraint>,
}

#[derive(Clone, Serialize, Debug)]
pub enum NumericConfig {
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        value: Option<Range>,
    },
    Decimal {
        value: Option<Range>,
    },
    Percent {
        value: Range,
    },
    Range {
        value: Range,
        default: f64,
        step: f64,
    },
}

#[derive(Clone, Serialize, Debug)]
pub struct Range {
    pub min: f64,
    pub max: f64,
}

/// Common trait to access a field's configuration
pub trait CommonFieldConfig {
    fn get_field_config(&self) -> &FieldConfig;
    fn get_field_config_mut(&mut self) -> &mut FieldConfig;

    /// `true` si la valeur du champ ne doit jamais être exposée.
    ///
    /// Point d'interrogation unique du framework : rendu, remplissage, journaux et
    /// audit passent tous par ici, plutôt que de comparer `field_type()` à la
    /// chaîne `"password"` chacun de leur côté.
    fn is_password(&self) -> bool {
        self.get_field_config().is_password()
    }
}

impl CommonFieldConfig for FieldConfig {
    fn get_field_config(&self) -> &FieldConfig {
        self
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        self
    }
}

pub trait FormField: CommonFieldConfig + DynClone + std::fmt::Debug + Send + Sync {
    // ========================================================================
    // GETTERS - Default implementation via CommonFieldConfig
    // ========================================================================

    fn name(&self) -> &str {
        &self.get_field_config().name
    }

    fn label(&self) -> &str {
        &self.get_field_config().label
    }

    fn value(&self) -> &str {
        &self.get_field_config().value
    }

    fn placeholder(&self) -> &str {
        &self.get_field_config().placeholder
    }

    fn field_type(&self) -> &str {
        &self.get_field_config().type_field
    }

    fn template_name(&self) -> &str {
        &self.get_field_config().template_name
    }

    fn error(&self) -> Option<&String> {
        self.get_field_config().error.as_ref()
    }

    fn required(&self) -> bool {
        self.get_field_config().is_required.choice
    }

    // ========================================================================
    // SETTERS - Default implementation via CommonFieldConfig
    // ========================================================================

    fn set_name(&mut self, name: &str) {
        self.get_field_config_mut().name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.get_field_config_mut().label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.get_field_config_mut().value = value.to_string();
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        self.get_field_config_mut().placeholder = placeholder.to_string();
    }

    fn set_error(&mut self, message: String) {
        let config = self.get_field_config_mut();
        config.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    /// Resets the field error (explicit equivalent of `set_error("")`)
    fn clear_error(&mut self) {
        self.get_field_config_mut().error = None;
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.get_field_config_mut().is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.get_field_config_mut()
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        self.get_field_config_mut().extra_context.insert(
            "readonly".to_string(),
            json!({
                "choice": readonly,
                "message": msg.map(|s| s.to_string())
            }),
        );
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        self.get_field_config_mut().extra_context.insert(
            "disabled".to_string(),
            json!({
                "choice": disabled,
                "message": msg.map(|s| s.to_string())
            }),
        );
    }

    /// Model-defined ceiling for max_size (file fields only). None for other field types.
    fn model_max_size(&self) -> Option<u64> {
        None
    }

    /// Overrides the effective max_size. Returns Err if it exceeds the model ceiling.
    fn set_max_size_bounded(
        &mut self,
        _size: crate::forms::fields::FileSize,
    ) -> Result<(), String> {
        Err("ce champ ne supporte pas max_size".to_string())
    }

    /// Field-type specific validation
    fn validate(&mut self) -> bool;

    /// Contexte de rendu commun à **tous** les champs.
    ///
    /// Chaque implémentation de [`FormField::render`] part de là et n'ajoute que
    /// ses variables propres. Les templates de `field_html/` consomment tous
    /// `field`, `readonly.choice` et `disabled.choice` : les oublier ne se voyait
    /// pas sous Tera 1, qui évaluait une variable absente à faux — l'attribut
    /// n'était alors jamais posé, sans la moindre erreur. Neuf rendus sur dix-huit
    /// étaient dans ce cas.
    fn base_context(&self) -> tera::Context {
        let mut context = tera::Context::new();
        context.insert("field", self.get_field_config());
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());
        context
    }

    /// HTML rendering of the field
    fn render(&self, tera: &ATera) -> Result<String, String>;

    /// Finalization (e.g., password hashing)
    fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn to_json_value(&self) -> Value {
        json!(self.get_field_config().value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.get_field_config().is_required)
    }

    fn to_json_readonly(&self) -> Value {
        self.get_field_config()
            .extra_context
            .get("readonly")
            .cloned()
            .unwrap_or_else(|| json!({"choice": false, "message": null}))
    }

    fn to_json_disabled(&self) -> Value {
        self.get_field_config()
            .extra_context
            .get("disabled")
            .cloned()
            .unwrap_or_else(|| json!({"choice": false, "message": null}))
    }

    fn to_json_attributes(&self) -> Value {
        let attrs: Vec<(&String, &String)> =
            self.get_field_config().html_attributes.iter().collect();
        let map: serde_json::Map<String, Value> = attrs
            .into_iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        Value::Object(map)
    }

    fn to_json_meta(&self) -> Value {
        json!({})
    }
}
