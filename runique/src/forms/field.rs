// --- Definitions communes pour les champs de formulaire ---
use crate::forms::manager::{Forms, ValidationError};
use async_trait::async_trait;
use sea_orm::DbErr;
use serde_json::Value;

use crate::utils::aliases::{ATera, StrMap};
use dyn_clone::DynClone;

pub trait FormField: DynClone + std::fmt::Debug + Send + Sync {
    // Getters
    fn name(&self) -> &str;
    fn label(&self) -> &str;
    fn value(&self) -> &str;
    fn placeholder(&self) -> &str;
    fn field_type(&self) -> &str;
    fn error(&self) -> Option<&String>;

    fn required(&self) -> bool {
        false
    }
    // Setters
    fn set_name(&mut self, name: &str);
    fn set_label(&mut self, label: &str);
    fn set_placeholder(&mut self, placeholder: &str);
    fn set_value(&mut self, value: &str);
    fn set_error(&mut self, message: String);
    fn set_readonly(&mut self, _readonly: bool, _msg: Option<&str>) {}
    fn set_disabled(&mut self, _disabled: bool, _msg: Option<&str>) {}
    fn set_required(&mut self, required: bool, msg: Option<&str>);
    fn set_html_attribute(&mut self, key: &str, value: &str);

    // Validation
    fn validate(&mut self) -> bool;

    fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }

    // Rendu
    fn render(&self, tera: &ATera) -> Result<String, String>;

    // Conversion JSON
    fn to_json_value(&self) -> Value {
        Value::String(self.value().to_string())
    }

    fn to_json_required(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": self.required(),
            "message": null
        })
    }

    fn to_json_readonly(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": false,
            "message": null
        })
    }

    fn to_json_disabled(&self) -> serde_json::Value {
        serde_json::json!({
            "choice": false,
            "message": null
        })
    }

    fn to_json_attributes(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    /// Métadonnées spécifiques au type de champ (ex: extensions, tailles pour FileField).
    /// Default retourne un objet vide — à override uniquement si le champ a des métadonnées.
    fn to_json_meta(&self) -> serde_json::Value {
        serde_json::json!({})
    }
}

dyn_clone::clone_trait_object!(FormField);

// Extrait des modifications à faire dans trait_form.rs

#[async_trait]
pub trait RuniqueForm: Sized + Send + Sync {
    fn register_fields(form: &mut Forms);
    fn from_form(form: Forms) -> Self;
    fn get_form(&self) -> &Forms;
    fn get_form_mut(&mut self) -> &mut Forms;

    async fn clean(&mut self) -> Result<(), StrMap> {
        Ok(())
    }

    async fn is_valid(&mut self) -> bool {
        // 1. Validation individuelle des champs (incluant CSRF)
        // Ici, le mot de passe est encore en CLAIR (format texte).
        let fields_valid = match self.get_form_mut().is_valid() {
            Ok(valid) => valid,
            Err(ValidationError::StackOverflow) => {
                self.get_form_mut().global_errors.push(
                    "Stack overflow détecté : récursion infinie dans la validation".to_string(),
                );
                return false;
            }
            Err(_) => return false,
        };

        if !fields_valid {
            return false;
        }

        // 2. Validation métier croisée (CLEAN)
        // C'est ICI que le dev peut comparer mdp1 == mdp2 en clair.
        match self.clean().await {
            Ok(_) => {
                // 3. FINALISATION (Hachage)
                // Si tout est valide, on transforme les données (ex: Argon2)
                if let Err(e) = self.get_form_mut().finalize() {
                    self.get_form_mut().global_errors.push(e);
                    return false;
                }
                true
            }
            Err(business_errors) => {
                let form = self.get_form_mut();
                for (name, msg) in business_errors {
                    if let Some(field) = form.fields.get_mut(&name) {
                        field.set_error(msg);
                    } else {
                        form.global_errors.push(msg);
                    }
                }
                false
            }
        }
    }

    fn database_error(&mut self, err: &DbErr) {
        self.get_form_mut().database_error(err);
    }

    // MODIFIÉ : Prend maintenant le token CSRF
    fn build(tera: ATera, csrf_token: &str) -> Self {
        let mut form = Forms::new(csrf_token); // PASSAGE DU TOKEN
        form.set_tera(tera);
        Self::register_fields(&mut form);
        Self::from_form(form)
    }

    async fn build_with_data(
        raw_data: &StrMap,
        tera: ATera,
        csrf_token: &str, //  AJOUT DU PARAMÈTRE
    ) -> Self {
        let mut form = Forms::new(csrf_token); // PASSAGE DU TOKEN

        form.set_tera(tera.clone());

        Self::register_fields(&mut form);

        form.fill(raw_data);

        let mut instance = Self::from_form(form);

        let _is_valid = instance.is_valid().await;

        instance
    }
}
