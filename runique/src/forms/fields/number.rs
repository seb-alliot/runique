//! Numeric fields: `NumericField` (integer, decimal) with min/max validation and precision.
use crate::forms::base::*;
use crate::utils::aliases::ATera;
use crate::utils::trad::{t, tf};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::json;

/// Numeric input (integer, decimal, float, percent, or range slider).
/// Construct with [`NumericField::integer`], [`::decimal`](NumericField::decimal),
/// [`::float`](NumericField::float), [`::percent`](NumericField::percent),
/// or [`::range`](NumericField::range).
#[derive(Clone, Serialize, Debug)]
pub struct NumericField {
    pub base: FieldConfig,
    pub config: NumericConfig,
    pub min_digits: Option<usize>,
    pub max_digits: Option<usize>,
    /// What the Rust field behind an integer can hold (`integer_in`); `i64`
    /// otherwise. Checked before the `min`/`max` the developer adds.
    #[serde(skip)]
    pub int_range: Option<(i128, i128)>,
}

impl CommonFieldConfig for NumericField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl NumericField {
    fn create(name: &str, type_field: &str, config: NumericConfig) -> Self {
        Self {
            base: FieldConfig::new(name, type_field, "base_number.html"),
            config,
            min_digits: None,
            max_digits: None,
            int_range: None,
        }
    }
    /// Constrains the number of decimal digits (e.g. `digits(2, 4)` for `12.34` to `12.3456`).
    pub fn digits(mut self, min: usize, max: usize) -> Self {
        self.min_digits = Some(min);
        self.max_digits = Some(max);
        self
    }
    /// Integer input (`i64` range). No decimal part allowed.
    pub fn integer(name: &str) -> Self {
        Self::create(
            name,
            "number",
            NumericConfig::Integer {
                min: None,
                max: None,
            },
        )
    }

    /// Integer input limited to `min..=max`: what the Rust field it's saved
    /// into can hold (`-128..=127` for an `i8`, `0..=u64::MAX` for a `u64`).
    pub fn integer_in(name: &str, min: i128, max: i128) -> Self {
        let mut field = Self::integer(name);
        field.int_range = Some((min, max));
        field
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }

    /// Sets the HTML `placeholder` attribute.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    /// Floating-point input (`f64`). Accepts `,` as decimal separator.
    pub fn float(name: &str) -> Self {
        Self::create(name, "number", NumericConfig::Float { value: None })
    }

    /// Decimal input (arbitrary precision via `rust_decimal`). Accepts `,` as decimal separator.
    pub fn decimal(name: &str) -> Self {
        Self::create(name, "number", NumericConfig::Decimal { value: None })
    }

    /// Percentage input. Valid range: `0.0–100.0`.
    pub fn percent(name: &str) -> Self {
        Self::create(
            name,
            "number",
            NumericConfig::Percent {
                value: Range {
                    min: 0.0,
                    max: 100.0,
                },
            },
        )
    }

    /// Range slider `<input type="range">`. `default` is the initial value.
    pub fn range(name: &str, min: f64, max: f64, default: f64) -> Self {
        let mut field = Self::create(
            name,
            "range",
            NumericConfig::Range {
                value: Range { min, max },
                default,
                step: 1.0,
            },
        );
        field.base.value = default.to_string();
        field
    }

    /// Minimum accepted value. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, val: f64, msg: &str) -> Self {
        match &mut self.config {
            NumericConfig::Integer { min, .. } => *min = Some(val as i64),
            NumericConfig::Float { value } | NumericConfig::Decimal { value, .. } => {
                if let Some(f) = value {
                    f.min = val;
                } else {
                    *value = Some(Range {
                        min: val,
                        max: f64::MAX,
                    });
                }
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                value.min = val;
            }
        }
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    /// Maximum accepted value. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, val: f64, msg: &str) -> Self {
        match &mut self.config {
            NumericConfig::Integer { max, .. } => *max = Some(val as i64),
            NumericConfig::Float { value } | NumericConfig::Decimal { value, .. } => {
                if let Some(f) = value {
                    f.max = val;
                } else {
                    *value = Some(Range {
                        min: f64::MIN,
                        max: val,
                    });
                }
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                value.max = val;
            }
        }
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    /// Step increment for range sliders.
    pub fn step(mut self, s: f64) -> Self {
        if let NumericConfig::Range { step, .. } = &mut self.config {
            *step = s;
        }
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }
}

impl NumericField {
    /// The message given to `min` / `max` for this bound, else the default.
    fn bound_message(&self, key: &str, default: impl FnOnce() -> String) -> String {
        self.base
            .extra_context
            .get(key)
            .and_then(|v| v.as_str())
            .map_or_else(default, str::to_string)
    }
}

// --- Trait Implementation ---
#[async_trait]
impl FormField for NumericField {
    fn bounds(&self) -> crate::forms::base::FieldBounds {
        use crate::forms::base::FieldBounds;
        match &self.config {
            NumericConfig::Integer { min, max } => FieldBounds {
                min_int: *min,
                max_int: *max,
                ..Default::default()
            },
            NumericConfig::Float { value } | NumericConfig::Decimal { value } => FieldBounds {
                min_float: value.as_ref().map(|r| r.min),
                max_float: value.as_ref().map(|r| r.max),
                ..Default::default()
            },
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => FieldBounds {
                min_float: Some(value.min),
                max_float: Some(value.max),
                ..Default::default()
            },
        }
    }

    fn set_type_bounds(&mut self, min: i128, max: i128) -> bool {
        if !matches!(self.config, NumericConfig::Integer { .. }) {
            return false;
        }
        self.int_range = Some((min, max));
        true
    }

    async fn validate(&mut self) -> bool {
        let val = self.base.value.trim();
        if self.base.is_required.choice && val.is_empty() {
            self.set_error(
                self.base
                    .is_required
                    .message
                    .clone()
                    .unwrap_or_else(|| t("forms.number_required").to_string()),
            );
            return false;
        }
        if val.is_empty() {
            return true;
        }

        let normalized = val.replace(',', ".");

        // --- STEP 1: Precision validation (digits) ---
        let current_digits = normalized
            .find('.')
            .map(|dot| normalized[dot.saturating_add(1)..].len())
            .unwrap_or(0);

        if current_digits < self.min_digits.unwrap_or(0) {
            self.set_error(tf("forms.precision_min", &[&self.min_digits.unwrap_or(0)]));
            return false;
        }
        if current_digits > self.max_digits.unwrap_or(usize::MAX) {
            self.set_error(tf(
                "forms.precision_max",
                &[&self.max_digits.unwrap_or(usize::MAX)],
            ));
            return false;
        }

        // --- STEP 2: Value bounds validation (min/max) ---
        let canonical = match &self.config {
            NumericConfig::Integer { min, max } => {
                let Ok(v) = normalized.parse::<i128>() else {
                    self.set_error(t("forms.integer_required").to_string());
                    return false;
                };
                let (lo, hi) = self.int_range.unwrap_or((i64::MIN.into(), i64::MAX.into()));
                let lo = min.map_or(lo, |m| lo.max(m.into()));
                let hi = max.map_or(hi, |m| hi.min(m.into()));
                if v < lo {
                    let msg = self.bound_message("min_message", || tf("forms.min_value", &[&lo]));
                    self.set_error(msg);
                    return false;
                }
                if v > hi {
                    let msg = self.bound_message("max_message", || tf("forms.max_value", &[&hi]));
                    self.set_error(msg);
                    return false;
                }
                v.to_string()
            }
            NumericConfig::Decimal { value, .. } | NumericConfig::Float { value } => {
                // A decimal is read the way it's saved (`Decimal`, exact, no
                // exponent), not as an `f64` that would accept `1e5` and round.
                let parsed = if matches!(self.config, NumericConfig::Decimal { .. }) {
                    rust_decimal::Decimal::from_str_exact(&normalized)
                        .ok()
                        .and_then(|d| rust_decimal::prelude::ToPrimitive::to_f64(&d))
                } else {
                    normalized.parse::<f64>().ok().filter(|v| v.is_finite())
                };
                let Some(v) = parsed else {
                    self.set_error(t("forms.number_invalid").to_string());
                    return false;
                };
                if let Some(f) = value.as_ref() {
                    let (min, max) = (f.min, f.max);
                    if v < min {
                        let msg =
                            self.bound_message("min_message", || tf("forms.min_value", &[&min]));
                        self.set_error(msg);
                        return false;
                    }
                    if v > max {
                        let msg =
                            self.bound_message("max_message", || tf("forms.max_value", &[&max]));
                        self.set_error(msg);
                        return false;
                    }
                }
                normalized
            }
            NumericConfig::Percent { value } | NumericConfig::Range { value, .. } => {
                let (min, max) = (value.min, value.max);
                match normalized.parse::<f64>() {
                    Ok(v) if v >= min && v <= max => normalized,
                    Ok(v) if v.is_finite() => {
                        let key = if v < min {
                            "min_message"
                        } else {
                            "max_message"
                        };
                        let msg = self.bound_message(key, || t("forms.number_invalid").to_string());
                        self.set_error(msg);
                        return false;
                    }
                    _ => {
                        self.set_error(t("forms.number_invalid").to_string());
                        return false;
                    }
                }
            }
        };
        // What gets saved is the value that was checked: the conversion after
        // validation parses this string again, without the `,` → `.` or the trim.
        self.base.value = canonical;
        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();
        context.insert("config", &self.config);

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
