use crate::forms::base::FormField;
use crate::forms::fields::TextField;
use crate::forms::generic::GenericField;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;

use crate::utils::aliases::{ATera, FieldsMap, JsonMap, OATera, StrMap};
use crate::utils::constante::{
    CONSTRAINT_REGEX, CSRF_TOKEN_KEY, FAILED_REGEX, FOR_KEY_REGEX, KEY_REGEX,
};

// Erreurs possibles lors de la validation du formulaire liée a la bdd
#[derive(Debug, Clone)]
pub enum ValidationError {
    StackOverflow,
    FieldValidation(StrMap),
    GlobalErrors(Vec<String>),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::StackOverflow => {
                write!(
                    f,
                    "Stack overflow détecté : récursion infinie dans la validation"
                )
            }
            ValidationError::FieldValidation(errors) => {
                write!(f, "Erreurs de validation : {:?}", errors)
            }
            ValidationError::GlobalErrors(errors) => {
                write!(f, "Erreurs globales : {}", errors.join(", "))
            }
        }
    }
}

impl std::error::Error for ValidationError {}

thread_local! {
    static VALIDATION_DEPTH: Cell<usize> = const { Cell::new(0) };
}

#[derive(Clone)]
pub struct Forms {
    pub fields: FieldsMap,
    pub tera: OATera,
    pub global_errors: Vec<String>,
    pub session_csrf_token: Option<String>,
    pub js_files: Vec<String>,
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
        let mut state = serializer.serialize_struct("Forms", 7)?;

        state.serialize_field("data", &self.data())?;
        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;
        state.serialize_field("cleaned_data", &self.data())?;
        state.serialize_field("js_files", &self.js_files)?;

        let rendered_html = match self.render() {
            Ok(h) => h,
            Err(e) => format!("<p style='color:red'>Erreur de rendu : {}</p>", e),
        };

        state.serialize_field("html", &rendered_html)?;
        let rendered_fields: HashMap<String, String> = self
            .fields
            .iter()
            .filter_map(|(name, field)| {
                let tera_instance = self.tera.as_ref()?;
                field
                    .render(tera_instance)
                    .ok()
                    .map(|html| (name.clone(), html))
            })
            .collect();
        state.serialize_field("rendered_fields", &rendered_fields)?;

        let fields_data: HashMap<String, serde_json::Value> = self
            .fields
            .iter()
            .enumerate()
            .map(|(index, (name, field))| {
                let mut field_map = serde_json::Map::new();
                field_map.insert("name".to_string(), json!(name));
                field_map.insert("label".to_string(), json!(field.label()));
                field_map.insert("field_type".to_string(), json!(field.field_type()));
                field_map.insert("template_name".to_string(), json!(field.template_name()));
                field_map.insert("value".to_string(), json!(field.value()));
                field_map.insert("placeholder".to_string(), json!(field.placeholder()));
                field_map.insert("index".to_string(), json!(index));

                field_map.insert("is_required".to_string(), field.to_json_required());
                field_map.insert("readonly".to_string(), field.to_json_readonly());
                field_map.insert("disabled".to_string(), field.to_json_disabled());
                field_map.insert("html_attributes".to_string(), field.to_json_attributes());
                field_map.insert("meta".to_string(), field.to_json_meta());
                if let Some(err) = field.error() {
                    field_map.insert("error".to_string(), json!(err));
                }
                (name.clone(), Value::Object(field_map))
            })
            .collect();

        state.serialize_field("fields", &fields_data)?;
        state.end()
    }
}

impl Forms {
    pub fn new(csrf_token: &str) -> Self {
        let mut fields: FieldsMap = IndexMap::new();

        // Créer le champ CSRF
        let mut csrf_field = TextField::create_csrf();
        csrf_field.set_value(csrf_token);

        fields.insert(
            CSRF_TOKEN_KEY.to_string(),
            Box::new(csrf_field) as Box<dyn FormField>,
        );

        Self {
            fields,
            tera: None,
            global_errors: Vec::new(),
            session_csrf_token: Some(csrf_token.to_string()),
            js_files: Vec::new(),
        }
    }

    fn render_js(&self, tera: &ATera) -> Result<String, String> {
        if self.js_files.is_empty() {
            return Ok(String::new());
        }

        let template_name = "js_files";

        if !tera.get_template_names().any(|name| name == template_name) {
            return Err(format!("Template manquant: {}", template_name));
        }

        let mut context = tera::Context::new();
        context.insert("js_files", &self.js_files);

        let result = tera
            .render(template_name, &context)
            .map_err(|e| format!("Erreur rendu JS: {}", e))?;

        Ok(result)
    }

    pub fn add_js(&mut self, file: &str) {
        // Validation stricte
        if !file.ends_with(".js") {
            panic!("add_js() accepte uniquement .js, reçu: '{}'", file);
        }
        // Stockage simple
        self.js_files.push(file.to_string());
    }
    /// La solution au "type annotations needed" :
    /// On force la conversion en GenericField ici même.
    pub fn field<T>(&mut self, field_template: &T)
    where
        T: FormField + Clone + Into<GenericField> + 'static,
    {
        let generic_instance: GenericField = field_template.clone().into();
        self.fields.insert(
            generic_instance.name().to_string(),
            Box::new(generic_instance),
        );
    }

    pub fn set_tera(&mut self, tera: ATera) {
        self.tera = Some(tera);
    }

    pub fn fill(&mut self, data: &StrMap) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }
    pub fn finalize(&mut self) -> Result<(), String> {
        for (name, field) in self.fields.iter_mut() {
            if let Err(e) = field.finalize() {
                return Err(format!(
                    "Erreur lors de la finalisation du champ '{}': {}",
                    name, e
                ));
            }
        }
        Ok(())
    }
    /// Valide le formulaire avec protection contre les stack overflows
    /// Retourne un Result pour permettre la propagation des erreurs
    pub fn is_valid(&mut self) -> Result<bool, ValidationError> {
        self.is_valid_with_depth(0)
    }

    // Implémentation interne avec compteur
    fn is_valid_with_depth(&mut self, depth: usize) -> Result<bool, ValidationError> {
        const MAX_DEPTH: usize = 20;

        if depth > MAX_DEPTH {
            return Err(ValidationError::StackOverflow);
        }

        // Validation CSRF
        if let Some(csrf_field) = self.fields.get_mut(CSRF_TOKEN_KEY) {
            let submitted_token = csrf_field.value().to_string();
            if let Some(session_token) = &self.session_csrf_token {
                if submitted_token.trim().is_empty() {
                    csrf_field.set_error("Token CSRF manquant".to_string());
                    return Ok(false);
                }
                if submitted_token != *session_token {
                    csrf_field.set_error("Token CSRF invalide".to_string());
                    return Ok(false);
                }
            }
        }

        // Validation normale
        let mut is_all_valid = true;
        for field in self.fields.values_mut() {
            if field.required() && field.value().trim().is_empty() {
                field.set_error("Ce champ est obligatoire".to_string());
                is_all_valid = false;
                continue;
            }

            // Si validate() peut être récursif, passe depth + 1
            if !field.validate() {
                is_all_valid = false;
            }
        }

        let result = is_all_valid && self.global_errors.is_empty();

        if !result {
            if !self.global_errors.is_empty() {
                return Err(ValidationError::GlobalErrors(self.global_errors.clone()));
            } else {
                return Err(ValidationError::FieldValidation(self.errors()));
            }
        }

        Ok(true)
    }

    pub fn has_errors(&self) -> bool {
        !self.global_errors.is_empty() || self.fields.values().any(|f| f.error().is_some())
    }

    pub fn data(&self) -> JsonMap {
        self.fields
            .iter()
            .map(|(name, field)| (name.clone(), field.to_json_value()))
            .collect()
    }

    pub fn errors(&self) -> StrMap {
        let mut errs: StrMap = self
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

        let js_html = self.render_js(tera_instance)?;

        if !js_html.is_empty() {
            html.push(js_html);
        }

        // 1. Render tous les fields
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

        // Gestion des erreurs d'unicité avec extraction automatique du champ
        if err_msg.contains("unique") || err_msg.contains("UNIQUE") || err_msg.contains("Duplicate")
        {
            let field_name = Self::extract_field_name(&err_msg);

            if let Some(field) = field_name {
                // Trouver le champ correspondant et lui attribuer l'erreur
                if let Some(form_field) = self.fields.get_mut(&field) {
                    let friendly_name = field.replace("_", " ");
                    form_field.set_error(format!("Ce {} est déjà utilisé.", friendly_name));
                } else {
                    // Si le champ n'existe pas dans le formulaire, erreur globale
                    self.global_errors
                        .push(format!("La valeur du champ '{}' est déjà utilisée.", field));
                }
            } else {
                // Erreur d'unicité mais impossible d'extraire le champ
                self.global_errors
                    .push("Une contrainte d'unicité a été violée.".to_string());
            }
        } else {
            // Autres erreurs de base de données
            self.global_errors.push(format!("Erreur DB: {}", err_msg));
        }
    }

    /// Extraire le nom du champ depuis différents formats d'erreur SQL
    fn extract_field_name(err_msg: &str) -> Option<String> {
        // 1. PostgreSQL: constraint name
        if let Some(cap) = CONSTRAINT_REGEX.captures(err_msg).ok()? {
            let constraint = cap.get(1)?.as_str();
            return Self::parse_constraint_name(constraint);
        }

        // 2. PostgreSQL: Key (field)=(value)
        if let Some(cap) = KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        // 3. SQLite: UNIQUE constraint failed: table.field
        if let Some(cap) = FAILED_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        // 4. MySQL: for key 'table.field'
        if let Some(cap) = FOR_KEY_REGEX.captures(err_msg).ok()? {
            return Some(cap.get(1)?.as_str().to_string());
        }

        None
    }

    /// Parser le nom de contrainte pour extraire le nom du champ
    fn parse_constraint_name(constraint: &str) -> Option<String> {
        let parts: Vec<&str> = constraint.split('_').collect();

        if parts.len() >= 3 {
            // Format: table_field_key ou table_field_idx
            let field_parts = &parts[1..parts.len() - 1];
            return Some(field_parts.join("_"));
        }

        None
    }

    pub fn add_value(&mut self, name: &str, value: &str) {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
        }
    }
}
