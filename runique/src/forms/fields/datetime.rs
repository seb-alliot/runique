use crate::forms::base::FieldConfig;
use crate::forms::field::FormField;
use crate::forms::options::BoolChoice;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::Serialize;
use serde_json::{json, Value};
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

    pub fn min(mut self, date: NaiveDate, msg: &str) -> Self {
        self.min_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("min_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn max(mut self, date: NaiveDate, msg: &str) -> Self {
        self.max_date = Some(date);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for DateField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn template_name(&self) -> &str {
        &self.base.template_name
    }
    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> &str {
        &self.base.value
    }

    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est obligatoire".into());
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
                self.set_error("Format de date invalide (attendu: YYYY-MM-DD)".into());
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
                    .unwrap_or_else(|| format!("Date minimale: {}", min));
                self.set_error(msg);
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
                    .unwrap_or_else(|| format!("Date maximale: {}", max));
                self.set_error(msg);
                return false;
            }
        }

        self.set_error("".into());
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
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
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
                .insert("min_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn max(mut self, time: NaiveTime, msg: &str) -> Self {
        self.max_time = Some(time);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for TimeField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn template_name(&self) -> &str {
        &self.base.template_name
    }
    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> &str {
        &self.base.value
    }

    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est obligatoire".into());
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
                self.set_error("Format de temps invalide (attendu: HH:MM)".into());
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
                    .unwrap_or_else(|| format!("Temps minimal: {}", min));
                self.set_error(msg);
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
                    .unwrap_or_else(|| format!("Temps maximal: {}", max));
                self.set_error(msg);
                return false;
            }
        }

        self.set_error("".into());
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
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
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
                .insert("min_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn max(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.max_datetime = Some(datetime);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for DateTimeField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn template_name(&self) -> &str {
        &self.base.template_name
    }
    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> &str {
        &self.base.value
    }

    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est obligatoire".into());
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
                self.set_error("Format de date/temps invalide (attendu: YYYY-MM-DDTHH:MM)".into());
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
                    .unwrap_or_else(|| format!("Date/temps minimal: {}", min));
                self.set_error(msg);
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
                    .unwrap_or_else(|| format!("Date/temps maximal: {}", max));
                self.set_error(msg);
                return false;
            }
        }

        self.set_error("".into());
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
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        json!(self.base.value)
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
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
                .insert("min_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn max_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.max_seconds = Some(seconds);
        if !msg.is_empty() {
            self.base
                .extra_context
                .insert("max_message".to_string(), msg.to_string());
        }
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.base.label = label.to_string();
        self
    }

    pub fn required(mut self, msg: &str) -> Self {
        self.set_required(true, Some(msg));
        self
    }
}

impl FormField for DurationField {
    fn name(&self) -> &str {
        &self.base.name
    }
    fn template_name(&self) -> &str {
        &self.base.template_name
    }
    fn label(&self) -> &str {
        &self.base.label
    }

    fn value(&self) -> &str {
        &self.base.value
    }

    fn placeholder(&self) -> &str {
        &self.base.placeholder
    }

    fn field_type(&self) -> &str {
        &self.base.type_field
    }

    fn required(&self) -> bool {
        self.base.is_required.choice
    }

    fn error(&self) -> Option<&String> {
        self.base.error.as_ref()
    }

    fn set_name(&mut self, name: &str) {
        self.base.name = name.to_string();
    }

    fn set_label(&mut self, label: &str) {
        self.base.label = label.to_string();
    }

    fn set_value(&mut self, value: &str) {
        self.base.value = value.to_string();
    }

    fn set_placeholder(&mut self, p: &str) {
        self.base.placeholder = p.to_string();
    }

    fn set_error(&mut self, message: String) {
        self.base.error = if message.is_empty() {
            None
        } else {
            Some(message)
        };
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        self.base.is_required = BoolChoice {
            choice: required,
            message: msg.map(|s| s.to_string()),
        };
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        self.base
            .html_attributes
            .insert(key.to_string(), value.to_string());
    }

    fn validate(&mut self) -> bool {
        let val = self.base.value.trim();

        if self.base.is_required.choice && val.is_empty() {
            let msg = self
                .base
                .is_required
                .message
                .clone()
                .unwrap_or_else(|| "Ce champ est obligatoire".into());
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
                self.set_error("Durée invalide (nombre de secondes attendu)".into());
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
                    .unwrap_or_else(|| format!("Durée minimale: {} secondes", min));
                self.set_error(msg);
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
                    .unwrap_or_else(|| format!("Durée maximale: {} secondes", max));
                self.set_error(msg);
                return false;
            }
        }

        self.set_error("".into());
        true
    }

    fn render(&self, tera: &Arc<Tera>) -> Result<String, String> {
        let mut context = Context::new();
        context.insert("field", &self.base);

        if let Some(min) = &self.min_seconds {
            context.insert("min_seconds", min);
        }
        if let Some(max) = &self.max_seconds {
            context.insert("max_seconds", max);
        }

        tera.render(&self.base.template_name, &context)
            .map_err(|e| e.to_string())
    }

    fn to_json_value(&self) -> Value {
        self.base
            .value
            .parse::<u64>()
            .map(|v| json!(v))
            .unwrap_or(json!(null))
    }

    fn to_json_required(&self) -> Value {
        json!(self.base.is_required)
    }

    fn to_json_attributes(&self) -> Value {
        json!(self.base.html_attributes)
    }
}
