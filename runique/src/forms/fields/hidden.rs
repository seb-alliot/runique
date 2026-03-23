use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::trad::{t, tf};
use serde::Serialize;
use std::sync::Arc;
use subtle::ConstantTimeEq;
use tera::{Context, Tera};

#[derive(Clone, Serialize, Debug)]
pub struct HiddenField {
    pub base: FieldConfig,
    /// Token de session attendu (pour validation CSRF)
    pub expected_value: Option<String>,
}

impl HiddenField {
    /// Constructeur spécifique pour un champ caché CSRF
    pub fn new_csrf() -> Self {
        Self {
            base: FieldConfig::new("csrf_token", "hidden", "csrf"),
            expected_value: None,
        }
    }

    /// Constructeur générique pour un champ caché
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "hidden", "base_string"),
            expected_value: None,
        }
    }

    /// Définit la valeur attendue pour la validation (token de session)
    pub fn set_expected_value(&mut self, expected: &str) {
        self.expected_value = Some(expected.to_string());
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }
}

impl CommonFieldConfig for HiddenField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for HiddenField {
    fn validate(&mut self) -> bool {
        // Pour un champ CSRF, vérifier que la valeur correspond à celle attendue
        if self.base.name == "csrf_token" {
            if let Some(expected) = &self.expected_value {
                if self.base.value.trim().is_empty() {
                    self.set_error(t("csrf.missing").to_string());
                    return false;
                }

                // ct_eq : comparaison constant-time — évite qu'un attaquant
                // devine le token octet par octet via le temps de réponse
                if !bool::from(self.base.value.as_bytes().ct_eq(expected.as_bytes())) {
                    self.set_error(t("csrf.invalid").to_string());
                    return false;
                }
            }
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("input_type", &self.base.type_field);
        context.insert("readonly", &serde_json::json!({"choice": false}));
        context.insert("disabled", &serde_json::json!({"choice": false}));

        tera.render(&self.base.template_name, &context)
            .map_err(|e| {
                tf(
                    "forms.finalize_error",
                    &[&self.base.template_name, &e.to_string()],
                )
                .to_string()
            })
    }
}
