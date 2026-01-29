use crate::forms::field::FormField;
use crate::forms::fields::TextField;
use crate::forms::generic::GenericField;
use indexmap::IndexMap;
use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::Cell;
use std::collections::HashMap;
use std::fmt;

use crate::aliases::{ATera, OATera};
use crate::constante::CSRF_TOKEN_KEY;

// Erreurs possibles lors de la validation du formulaire liée a la bdd
#[derive(Debug, Clone)]
pub enum ValidationError {
    StackOverflow,
    FieldValidation(HashMap<String, String>),
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
    pub fields: IndexMap<String, Box<dyn FormField>>,
    pub tera: OATera,
    pub global_errors: Vec<String>,
    pub session_csrf_token: Option<String>,
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
        let mut state = serializer.serialize_struct("Forms", 6)?;

        state.serialize_field("data", &self.data())?;
        state.serialize_field("errors", &self.errors())?;
        state.serialize_field("global_errors", &self.global_errors)?;
        state.serialize_field("cleaned_data", &self.data())?;

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

                field_map.insert("is_required".to_string(), field.to_json_required());
                field_map.insert("readonly".to_string(), field.to_json_readonly());
                field_map.insert("disabled".to_string(), field.to_json_disabled());
                field_map.insert("html_attributes".to_string(), field.to_json_attributes());
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
        let mut fields: IndexMap<String, Box<dyn FormField>> = IndexMap::new();

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
            session_csrf_token: None,
        }
    }

    // Méthode helper pour créer sans CSRF (pour les cas où ce n'est pas nécessaire)
    pub fn new_without_csrf() -> Self {
        Self {
            fields: IndexMap::new(),
            tera: None,
            global_errors: Vec::new(),
            session_csrf_token: None,
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

    pub fn set_tera(&mut self, tera: ATera) {
        self.tera = Some(tera);
    }

    pub fn fill(&mut self, data: &HashMap<String, String>) {
        for field in self.fields.values_mut() {
            if let Some(value) = data.get(field.name()) {
                field.set_value(value);
            }
        }
    }

    /// Valide le formulaire avec protection contre les stack overflows
    /// Retourne un Result pour permettre la propagation des erreurs
    pub async fn is_valid(&mut self) -> Result<bool, ValidationError> {
        const MAX_VALIDATION_DEPTH: usize = 20;

        // Vérifier la profondeur de récursion
        let current_depth = VALIDATION_DEPTH.with(|d| d.get());

        if current_depth > MAX_VALIDATION_DEPTH {
            // Reset le compteur pour éviter de bloquer les prochaines requêtes
            VALIDATION_DEPTH.with(|d| d.set(0));
            return Err(ValidationError::StackOverflow);
        }

        // Incrémenter le compteur
        VALIDATION_DEPTH.with(|d| d.set(current_depth + 1));

        // VALIDATION SPÉCIALE POUR LE CSRF
        if let Some(csrf_field) = self.fields.get_mut(CSRF_TOKEN_KEY) {
            let submitted_token = csrf_field.value().to_string();

            if let Some(session_token) = &self.session_csrf_token {
                if submitted_token.trim().is_empty() {
                    csrf_field.set_error("Token CSRF manquant".to_string());
                    VALIDATION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                    return Ok(false);
                }

                if submitted_token != *session_token {
                    csrf_field.set_error("Token CSRF invalide".to_string());
                    VALIDATION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
                    return Ok(false);
                }
            }
        }

        // Validation normale des champs
        let mut is_all_valid = true;
        for field in self.fields.values_mut() {
            if field.required() && field.value().trim().is_empty() {
                field.set_error("Ce champ est obligatoire".to_string());
                is_all_valid = false;
                continue;
            }

            if !field.validate() {
                is_all_valid = false;
            }
        }

        let result = is_all_valid && self.global_errors.is_empty();

        // Décrémenter le compteur avant de retourner
        VALIDATION_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));

        // Si pas valide, retourner les erreurs appropriées
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
        // PostgreSQL français: contrainte unique « users_username_key »
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

        // PostgreSQL: Key (username)=(value)
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

    /// Parser le nom de contrainte pour extraire le nom du champ
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

    pub fn add_value(&mut self, name: &str, value: &str) {
        if let Some(field) = self.fields.get_mut(name) {
            field.set_value(value);
        }
    }
}
