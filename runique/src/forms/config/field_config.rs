//! A field's configuration: `FieldConfig`, common to every field, and what
//! the text and numeric fields add to it.
use super::{BoolChoice, Limit};
use crate::utils::aliases::{JsonMap, StrMap};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Common configuration for a form field (name, label, value, error, HTML attributes).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FieldConfig {
    pub name: String,
    pub label: String,
    pub value: String,
    pub placeholder: String,
    pub is_required: BoolChoice,
    /// Shown, never written: the field is sent, so its value could be forged.
    #[serde(default)]
    pub readonly: BoolChoice,
    /// Shown, never written: the browser doesn't send the field.
    #[serde(default)]
    pub disabled: BoolChoice,
    pub error: Option<String>,
    pub type_field: String,
    pub html_attributes: StrMap,
    pub template_name: String,
    pub extra_context: JsonMap,
    /// Field whose value must **never** leak out: not in the rendered widget,
    /// not in an audit trail, not in an admin view.
    ///
    /// Private, with no way to turn it back off: the only way to set it is
    /// [`FieldConfig::mark_password`] — there's no way to unset it. Read via
    /// [`FieldConfig::is_password`], which also returns `true` for any field of
    /// type `password` — so a sensitive field stays protected even if no one
    /// thought to set the flag.
    #[serde(default)]
    is_password: bool,
    /// The column this field is stored in, set by
    /// [`FormField::apply_column_type`](crate::forms::traits::form_field::FormField::apply_column_type). `None`: a field no column was given
    /// to (a form without a model, a confirmation, a captcha).
    #[serde(skip)]
    pub column_type: Option<sea_orm::sea_query::ColumnType>,
    /// Name of the form this field belongs to, set by `try_form`: prefixes the
    /// field's HTML `id` (`id_{form}-{field}`), unique across forms on a page.
    #[serde(skip)]
    pub form_name: Option<&'static str>,
}

impl FieldConfig {
    /// Creates a config with the given name, semantic `type_field` (e.g. `"text"`,
    /// `"password"`), and Tera `template_name`. `is_password` is derived from
    /// `type_field` here, so a `"password"` type is always protected from the start.
    pub fn new(name: &str, type_field: &str, template_name: &str) -> Self {
        Self {
            name: name.to_string(),
            label: String::new(),
            value: String::new(),
            placeholder: String::new(),
            is_required: BoolChoice::default(),
            readonly: BoolChoice::default(),
            disabled: BoolChoice::default(),
            error: None,
            type_field: type_field.to_string(),
            html_attributes: HashMap::new(),
            template_name: template_name.to_string(),
            extra_context: HashMap::new(),
            // Derived from the type right at construction: any `password` field,
            // whatever constructor was used, is born protected. The flag can
            // therefore never be missed by omission.
            is_password: type_field == "password",
            column_type: None,
            form_name: None,
        }
    }

    /// `true` if this field's value must never be exposed.
    ///
    /// The type wins over the flag: even if the field was built by hand without
    /// going through `mark_password`, a `type_field` of `password` is enough to
    /// protect it.
    #[must_use]
    pub fn is_password(&self) -> bool {
        self.is_password || self.type_field == "password"
    }

    /// Marks the field as carrying a secret (API key, token…).
    ///
    /// One-way, on purpose: there is no reverse operation. A field declared
    /// sensitive can't stop being sensitive along the way, which rules out any
    /// code path accidentally "unmasking" it.
    pub fn mark_password(&mut self) {
        self.is_password = true;
    }
}

/// Length constraints for text-like fields (min/max character length).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TextConfig {
    pub max_length: Option<Limit<u32>>,
    pub min_length: Option<Limit<u32>>,
}

/// Per-kind numeric validation state. Which variant a field holds depends on
/// its semantic (integer, float, decimal, percent, or a stepped range slider);
/// [`NumericField::min`](crate::forms::fields::NumericField::min) and `::max`
/// match on it to update the right bound representation.
#[derive(Clone, Serialize, Debug)]
pub enum NumericConfig {
    Integer {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        value: Option<MinMax>,
    },
    Decimal {
        value: Option<MinMax>,
    },
    /// Bounded `0.0..=100.0` by default (see [`NumericField::percent`](crate::forms::fields::NumericField::percent)).
    Percent {
        value: MinMax,
    },
    /// Backs an `<input type="range">`; `default` is the initial value and
    /// `step` the increment.
    Range {
        value: MinMax,
        default: f64,
        step: f64,
    },
}

/// The bounds a field accepts: lengths for text, values for numbers. Read
/// after `customize` to check it didn't loosen what the model declares.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct FieldBounds {
    pub min_length: Option<u32>,
    pub max_length: Option<u32>,
    pub min_int: Option<i64>,
    pub max_int: Option<i64>,
    pub min_float: Option<f64>,
    pub max_float: Option<f64>,
    /// A decimal's digits after the point.
    pub max_decimal_places: Option<usize>,
    /// A decimal's digits before the point.
    pub max_integer_digits: Option<usize>,
}

/// An inclusive `min..=max` bound used by [`NumericConfig`] variants.
#[derive(Clone, Serialize, Debug)]
pub struct MinMax {
    pub min: f64,
    pub max: f64,
}
