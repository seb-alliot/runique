//! Form validation: `ValidationError` and `FormValidator` with error accumulation.
use crate::utils::{
    aliases::{FieldsMap, StrMap},
    trad::t,
};
use std::fmt;

/// Error returned when form validation fails to complete or produces errors.
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// Validation recursed past the maximum allowed depth (re-entrant
    /// `Forms::validate` calls) and was aborted as a safety guard.
    StackOverflow,
    /// Per-field validation errors, keyed by field name.
    FieldValidation(StrMap),
    /// Errors that apply to the form as a whole rather than a specific field.
    GlobalErrors(Vec<String>),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::StackOverflow => {
                write!(f, "{}", t("forms.validation_overflow"))
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

/// Stateless helper that validates a form's fields and reports resulting errors.
pub struct FormValidator;

impl FormValidator {
    pub(crate) async fn validate_fields(
        fields: &mut FieldsMap,
        global_errors: &[String],
    ) -> Result<bool, ValidationError> {
        let log_validate = crate::utils::runique_log::get_log()
            .forms
            .as_ref()
            .and_then(|f| f.validate);
        let mut is_all_valid = true;

        for field in fields.values_mut() {
            if field.required()
                && field.value().trim().is_empty()
                && field.field_type() != "checkbox"
            {
                field.set_error(t("forms.required").into_owned());
                is_all_valid = false;
                if let Some(level) = log_validate {
                    crate::runique_log!(level, field = %field.name(), "required field empty");
                }
                continue;
            }
            let valid = field.validate().await;
            if let Some(level) = log_validate {
                crate::runique_log!(
                    level,
                    field = %field.name(),
                    valid = valid,
                    error = ?field.error(),
                    "validate"
                );
            }
            if !valid {
                is_all_valid = false;
            }
        }

        let result = is_all_valid && global_errors.is_empty();
        if let Some(level) = log_validate {
            crate::runique_log!(
                level,
                ok = result,
                global_errors = global_errors.len(),
                "validate_fields result"
            );
        }

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

    /// Returns whether any global error is present or any field currently has an error set.
    pub fn has_errors(fields: &FieldsMap, global_errors: &[String]) -> bool {
        !global_errors.is_empty() || fields.values().any(|f| f.error().is_some())
    }

    /// Collects all field errors into a map keyed by field name, plus a
    /// `"global"` entry joining `global_errors` with `" | "` if any are present.
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
