//! Form validation: `ValidationError` and `FormValidator` with error accumulation.
use crate::utils::{
    aliases::{FieldsMap, StrMap},
    trad::t,
};
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

pub struct FormValidator;

impl FormValidator {
    pub(crate) fn validate_fields(
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
                    if crate::utils::env::is_debug() {
                        eprintln!(
                            "[runique:form] validate  {} — required field empty",
                            field.name()
                        );
                    }
                }
                continue;
            }
            let valid = field.validate();
            if let Some(level) = log_validate {
                crate::runique_log!(
                    level,
                    field = %field.name(),
                    valid = valid,
                    error = ?field.error(),
                    "validate"
                );
                if crate::utils::env::is_debug() {
                    eprintln!(
                        "[runique:form] validate  {} valid={} error={:?}",
                        field.name(),
                        valid,
                        field.error()
                    );
                }
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
            if crate::utils::env::is_debug() {
                eprintln!(
                    "[runique:form] validate result  ok={} global_errors={}",
                    result,
                    global_errors.len()
                );
            }
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
