use crate::forms::base::{CommonFieldConfig, FieldConfig, FormField};
use crate::utils::trad::{t, tf};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tera::{Context, Tera};

/// DateField - Champ de date (format: YYYY-MM-DD)
#[derive(Clone, Serialize, Debug)]
pub struct DateField {
    pub base: FieldConfig,
    pub min_date: Option<NaiveDate>,
    pub max_date: Option<NaiveDate>,
}

impl DateField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "date", "base_datetime"),
            min_date: None,
            max_date: None,
        }
    }

    pub fn placeholder(mut self, p: &str) -> Self {
        self.set_placeholder(p);
        self
    }

    pub fn min(mut self, date: NaiveDate, msg: &str) -> Self {
        self.min_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    pub fn max(mut self, date: NaiveDate, msg: &str) -> Self {
        self.max_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

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

        // Parser la date
        let date = match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                self.set_error(t("forms.date_invalid").to_string());
                return false;
            }
        };

        // Vérifier min
        if let Some(min) = self.min_date {
            if date < min {
                let msg = self
                    .base
                    .extra_context
                    .get("min_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.date_too_old", &[&min])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
        }

        // Vérifier max
        if let Some(max) = self.max_date {
            if date > max {
                let msg = self
                    .base
                    .extra_context
                    .get("max_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.date_too_far", &[&max])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
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

/// TimeField - Champ de temps (format: HH:MM)
#[derive(Clone, Serialize, Debug)]
pub struct TimeField {
    pub base: FieldConfig,
    pub min_time: Option<NaiveTime>,
    pub max_time: Option<NaiveTime>,
}

impl TimeField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "time", "base_datetime"),
            min_time: None,
            max_time: None,
        }
    }

    pub fn min(mut self, time: NaiveTime, msg: &str) -> Self {
        self.min_time = Some(time);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    pub fn max(mut self, time: NaiveTime, msg: &str) -> Self {
        self.max_time = Some(time);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

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

        // Parser le temps
        let time = match NaiveTime::parse_from_str(val, "%H:%M") {
            Ok(t) => t,
            Err(_) => {
                self.set_error(t("forms.time_invalid").to_string());
                return false;
            }
        };

        // Vérifier min
        if let Some(min) = self.min_time {
            if time < min {
                let msg = self
                    .base
                    .extra_context
                    .get("min_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.time_too_old", &[&min])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
        }

        // Vérifier max
        if let Some(max) = self.max_time {
            if time > max {
                let msg = self
                    .base
                    .extra_context
                    .get("max_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.time_too_far", &[&max])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
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

/// DateTimeField - Champ de date et temps combiné
#[derive(Clone, Serialize, Debug)]
pub struct DateTimeField {
    pub base: FieldConfig,
    pub min_datetime: Option<NaiveDateTime>,
    pub max_datetime: Option<NaiveDateTime>,
}

impl DateTimeField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "datetime-local", "base_datetime"),
            min_datetime: None,
            max_datetime: None,
        }
    }

    pub fn min(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.min_datetime = Some(datetime);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    pub fn max(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.max_datetime = Some(datetime);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

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

        // Parser le datetime (format ISO: YYYY-MM-DDTHH:MM)
        let datetime = match NaiveDateTime::parse_from_str(val, "%Y-%m-%dT%H:%M") {
            Ok(dt) => dt,
            Err(_) => {
                self.set_error(t("forms.date_invalid").to_string());
                return false;
            }
        };

        // Vérifier min
        if let Some(min) = self.min_datetime {
            if datetime < min {
                let msg = self
                    .base
                    .extra_context
                    .get("min_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.datetime_too_old", &[&min])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
        }

        // Vérifier max
        if let Some(max) = self.max_datetime {
            if datetime > max {
                let msg = self
                    .base
                    .extra_context
                    .get("max_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.datetime_too_far", &[&max])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
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

/// DurationField - Champ pour saisir une durée (en secondes)
#[derive(Clone, Serialize, Debug)]
pub struct DurationField {
    pub base: FieldConfig,
    pub min_seconds: Option<u64>,
    pub max_seconds: Option<u64>,
}

impl DurationField {
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "number", "base_datetime"),
            min_seconds: None,
            max_seconds: None,
        }
    }

    pub fn min_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.min_seconds = Some(seconds);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), json!(msg));
        }
        self
    }

    pub fn max_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.max_seconds = Some(seconds);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), json!(msg));
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

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

        // Parser la durée (en secondes)
        let seconds = match val.parse::<u64>() {
            Ok(s) => s,
            Err(_) => {
                self.set_error(t("forms.duration_invalid").to_string());
                return false;
            }
        };

        // Vérifier min
        if let Some(min) = self.min_seconds {
            if seconds < min {
                let msg = self
                    .base
                    .extra_context
                    .get("min_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.duration_too_short", &[&min])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
        }

        // Vérifier max
        if let Some(max) = self.max_seconds {
            if seconds > max {
                let msg = self
                    .base
                    .extra_context
                    .get("max_message")
                    .cloned()
                    .unwrap_or_else(|| json!(tf("forms.duration_too_long", &[&max])));
                self.set_error(msg.as_str().unwrap_or_default().to_string());
                return false;
            }
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
