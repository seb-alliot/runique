//! Date/time fields: `DateField`, `TimeField`, `DateTimeField` with min/max validation.
use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::trad::{t, tf};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tera::{Context, Tera};

/// Date input (`<input type="date">`). Validates `YYYY-MM-DD` format with optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct DateField {
    pub base: FieldConfig,
    pub min_date: Option<NaiveDate>,
    pub max_date: Option<NaiveDate>,
}

impl DateField {
    /// Creates a date field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "date", "base_datetime"),
            min_date: None,
            max_date: None,
        }
    }

    /// Sets the HTML `placeholder` attribute.
    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    /// Minimum accepted date. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, date: NaiveDate, msg: &str) -> Self {
        self.min_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    /// Maximum accepted date. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, date: NaiveDate, msg: &str) -> Self {
        self.max_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

impl CommonFieldConfig for DateField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for DateField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            return true;
        }

        // Parse the date
        let date = match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                self.set_error(t("forms.date_invalid").to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = self.min_date
            && date < min
        {
            let msg = self
                .base
                .extra_context
                .get("min_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.date_too_old", &[&min])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        // Check max
        if let Some(max) = self.max_date
            && date > max
        {
            let msg = self
                .base
                .extra_context
                .get("max_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.date_too_far", &[&max])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        if let Some(min) = &self.min_date {
            context.insert("min_date", &min.format("%Y-%m-%d").to_string());
        }
        if let Some(max) = &self.max_date {
            context.insert("max_date", &max.format("%Y-%m-%d").to_string());
        }

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

/// Time input (`<input type="time">`). Accepts `HH:MM` (HTML) or `HH:MM:SS` (database). Optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct TimeField {
    pub base: FieldConfig,
    pub min_time: Option<NaiveTime>,
    pub max_time: Option<NaiveTime>,
}

impl TimeField {
    /// Creates a time field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "time", "base_datetime"),
            min_time: None,
            max_time: None,
        }
    }

    /// Minimum accepted time. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, time: NaiveTime, msg: &str) -> Self {
        self.min_time = Some(time);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    /// Maximum accepted time. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, time: NaiveTime, msg: &str) -> Self {
        self.max_time = Some(time);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

impl CommonFieldConfig for TimeField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}
impl FormField for TimeField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            return true;
        }

        // Parse the time — accept both HH:MM (HTML input) and HH:MM:SS (PostgreSQL)
        let time = match NaiveTime::parse_from_str(val, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(val, "%H:%M:%S"))
        {
            Ok(t) => t,
            Err(_) => {
                self.set_error(t("forms.time_invalid").to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = self.min_time
            && time < min
        {
            let msg = self
                .base
                .extra_context
                .get("min_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.time_too_old", &[&min])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        // Check max
        if let Some(max) = self.max_time
            && time > max
        {
            let msg = self
                .base
                .extra_context
                .get("max_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.time_too_far", &[&max])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        if let Some(min) = &self.min_time {
            context.insert("min_time", &min.format("%H:%M").to_string());
        }
        if let Some(max) = &self.max_time {
            context.insert("max_time", &max.format("%H:%M").to_string());
        }

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

/// Combined date and time input (`<input type="datetime-local">`). Accepts `YYYY-MM-DDTHH:MM`. Optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct DateTimeField {
    pub base: FieldConfig,
    pub min_datetime: Option<NaiveDateTime>,
    pub max_datetime: Option<NaiveDateTime>,
}

impl DateTimeField {
    /// Creates a datetime-local field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "datetime-local", "base_datetime"),
            min_datetime: None,
            max_datetime: None,
        }
    }

    /// Minimum accepted datetime. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.min_datetime = Some(datetime);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    /// Maximum accepted datetime. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.max_datetime = Some(datetime);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}
impl CommonFieldConfig for DateTimeField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for DateTimeField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            return true;
        }

        // Parse the datetime — accept HTML (YYYY-MM-DDTHH:MM) and ISO with seconds only
        let datetime = match NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M")
            .or_else(|_| NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M:%S"))
        {
            Ok(dt) => dt,
            Err(_) => {
                self.set_error(t("forms.date_invalid").to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = self.min_datetime
            && datetime < min
        {
            let msg = self
                .base
                .extra_context
                .get("min_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.datetime_too_old", &[&min])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        // Check max
        if let Some(max) = self.max_datetime
            && datetime > max
        {
            let msg = self
                .base
                .extra_context
                .get("max_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.datetime_too_far", &[&max])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        if let Some(min) = &self.min_datetime {
            context.insert("min_datetime", &min.format("%Y-%m-%dT%H:%M").to_string());
        }
        if let Some(max) = &self.max_datetime {
            context.insert("max_datetime", &max.format("%Y-%m-%dT%H:%M").to_string());
        }

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

/// Duration input stored as seconds (`u64`). Optional min/max bounds in seconds.
#[derive(Clone, Serialize, Debug)]
pub struct DurationField {
    pub base: FieldConfig,
    pub min_seconds: Option<u64>,
    pub max_seconds: Option<u64>,
}

impl DurationField {
    /// Creates a duration field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "number", "base_datetime"),
            min_seconds: None,
            max_seconds: None,
        }
    }

    /// Minimum duration in seconds. `msg` overrides the default error (pass `""` for default).
    pub fn min_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.min_seconds = Some(seconds);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    /// Maximum duration in seconds. `msg` overrides the default error (pass `""` for default).
    pub fn max_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.max_seconds = Some(seconds);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    /// Overrides the auto-generated label.
    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    /// Marks the field as required (empty value fails validation).
    pub fn required(mut self) -> Self {
        self.set_required(true, None);
        self
    }
}

impl CommonFieldConfig for DurationField {
    fn get_field_config(&self) -> &FieldConfig {
        &self.base
    }

    fn get_field_config_mut(&mut self) -> &mut FieldConfig {
        &mut self.base
    }
}

impl FormField for DurationField {
    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| t("forms.required").to_string());
            self.set_error(msg);
            return false;
        }

        if val.is_empty() {
            return true;
        }

        // Parse the duration (in seconds)
        let seconds = match val.parse::<u64>() {
            Ok(s) => s,
            Err(_) => {
                self.set_error(t("forms.duration_invalid").to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = self.min_seconds
            && seconds < min
        {
            let msg = self
                .base
                .extra_context
                .get("min_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.duration_too_short", &[&min])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        // Check max
        if let Some(max) = self.max_seconds
            && seconds > max
        {
            let msg = self
                .base
                .extra_context
                .get("max_message")
                .cloned()
                .unwrap_or_else(|| json!(tf("forms.duration_too_long", &[&max])));
            self.set_error(msg.as_str().unwrap_or_default().to_string());
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);
        context.insert("readonly", &self.to_json_readonly());
        context.insert("disabled", &self.to_json_disabled());

        if let Some(min) = &self.min_seconds {
            context.insert("min_seconds", min);
        }
        if let Some(max) = &self.max_seconds {
            context.insert("max_seconds", max);
        }

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
