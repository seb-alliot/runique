use crate::utils::aliases::{FieldsMap, StrMap};
use std::fmt;

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
                    "Stack overflow detected: infinite recursion in validation"
                )
            }
            ValidationError::FieldValidation(errors) => {
                write!(f, "Validation errors: {:?}", errors)
            }
            ValidationError::GlobalErrors(errors) => {
                write!(f, "Global errors: {}", errors.join(", "))
            }
        }
    }
}

impl std::error::Error for ValidationError {}

pub struct FormValidator;

impl FormValidator {
    pub(crate) fn validate_fields(
        fields: &mut FieldsMap,
        global_errors: &[String],
    ) -> Result<bool, ValidationError> {
        let mut is_all_valid = true;

        for field in fields.values_mut() {
            if field.required() && field.value().trim().is_empty() {
                field.set_error("This field is required".to_string());
                is_all_valid = false;
                continue;
            }
            if !field.validate() {
                is_all_valid = false;
            }
        }

        let result = is_all_valid && global_errors.is_empty();

        if !result {
            if !global_errors.is_empty() {
                return Err(ValidationError::GlobalErrors(global_errors.to_vec()));
            } else {
                let errors: StrMap = fields
                    .iter()
                    .filter_map(|(name, field)| {
                        field.error().map(|err| (name.clone(), err.clone()))
                    })
                    .collect();
                return Err(ValidationError::FieldValidation(errors));
            }
        }

        Ok(true)
    }

    pub fn has_errors(fields: &FieldsMap, global_errors: &[String]) -> bool {
        !global_errors.is_empty() || fields.values().any(|f| f.error().is_some())
    }

    pub fn collect_errors(fields: &FieldsMap, global_errors: &[String]) -> StrMap {
        let mut errs: StrMap = fields
            .iter()
            .filter_map(|(name, field)| field.error().map(|err| (name.clone(), err.clone())))
            .collect();

        if !global_errors.is_empty() {
            errs.insert("global".to_string(), global_errors.join(" | "));
        }
        errs
    }
}
