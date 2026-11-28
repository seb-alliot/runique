//! Date/time fields: `DateField`, `TimeField`, `DateTimeField` with min/max validation.
use crate::forms::config::Limit;
use crate::forms::config::field_config::FieldConfig;
use crate::forms::traits::form_field::FormField;
use crate::translation::{t, tf};
use crate::utils::aliases::ATera;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use serde::Serialize;

/// Date input (`<input type="date">`). Validates `YYYY-MM-DD` format with optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct DateField {
    pub base: FieldConfig,
    pub min_date: Option<Limit<NaiveDate>>,
    pub max_date: Option<Limit<NaiveDate>>,
}

impl DateField {
    /// Creates a date field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "date", "base_datetime.html"),
            min_date: None,
            max_date: None,
        }
    }

    /// Minimum accepted date. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, date: NaiveDate, msg: &str) -> Self {
        self.min_date = Some(Limit::new(date, msg));
        self
    }

    /// Maximum accepted date. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, date: NaiveDate, msg: &str) -> Self {
        self.max_date = Some(Limit::new(date, msg));
        self
    }
}

field_base!(DateField, label, required, placeholder);

#[async_trait]
impl FormField for DateField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::data::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::Date,
        )
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required(runique_i18n::keys::forms::error::REQUIRED) {
            return false;
        }
        let val = self.base.value.trim();

        if val.is_empty() {
            return true;
        }

        // Parse the date
        let date = match NaiveDate::parse_from_str(val, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => {
                self.set_error(t(runique_i18n::keys::forms::error::DATE_INVALID).to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = &self.min_date
            && date < min.value
        {
            let msg = min.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DATE_TOO_OLD,
                    &[&min.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        // Check max
        if let Some(max) = &self.max_date
            && date > max.value
        {
            let msg = max.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DATE_TOO_FAR,
                    &[&max.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        if let Some(min) = &self.min_date {
            context.insert("min_date", &min.value.format("%Y-%m-%d").to_string());
        }
        if let Some(max) = &self.max_date {
            context.insert("max_date", &max.value.format("%Y-%m-%d").to_string());
        }

        self.render_template(tera, &context)
    }
}

/// Time input (`<input type="time">`). Accepts `HH:MM` (HTML) or `HH:MM:SS` (database). Optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct TimeField {
    pub base: FieldConfig,
    pub min_time: Option<Limit<NaiveTime>>,
    pub max_time: Option<Limit<NaiveTime>>,
}

impl TimeField {
    /// Creates a time field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "time", "base_datetime.html"),
            min_time: None,
            max_time: None,
        }
    }

    /// Minimum accepted time. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, time: NaiveTime, msg: &str) -> Self {
        self.min_time = Some(Limit::new(time, msg));
        self
    }

    /// Maximum accepted time. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, time: NaiveTime, msg: &str) -> Self {
        self.max_time = Some(Limit::new(time, msg));
        self
    }
}

field_base!(TimeField, label, required);
#[async_trait]
impl FormField for TimeField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::data::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::Time,
        )
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required(runique_i18n::keys::forms::error::REQUIRED) {
            return false;
        }
        let val = self.base.value.trim();

        if val.is_empty() {
            return true;
        }

        // Parse the time — accept both HH:MM (HTML input) and HH:MM:SS (PostgreSQL)
        let time = match NaiveTime::parse_from_str(val, "%H:%M")
            .or_else(|_| NaiveTime::parse_from_str(val, "%H:%M:%S"))
        {
            Ok(t) => t,
            Err(_) => {
                self.set_error(t(runique_i18n::keys::forms::error::TIME_INVALID).to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = &self.min_time
            && time < min.value
        {
            let msg = min.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::TIME_TOO_OLD,
                    &[&min.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        // Check max
        if let Some(max) = &self.max_time
            && time > max.value
        {
            let msg = max.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::TIME_TOO_FAR,
                    &[&max.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        if let Some(min) = &self.min_time {
            context.insert("min_time", &min.value.format("%H:%M").to_string());
        }
        if let Some(max) = &self.max_time {
            context.insert("max_time", &max.value.format("%H:%M").to_string());
        }

        self.render_template(tera, &context)
    }
}

/// Combined date and time input (`<input type="datetime-local">`). Accepts `YYYY-MM-DDTHH:MM`. Optional min/max bounds.
#[derive(Clone, Serialize, Debug)]
pub struct DateTimeField {
    pub base: FieldConfig,
    pub min_datetime: Option<Limit<NaiveDateTime>>,
    pub max_datetime: Option<Limit<NaiveDateTime>>,
}

impl DateTimeField {
    /// Creates a datetime-local field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "datetime-local", "base_datetime.html"),
            min_datetime: None,
            max_datetime: None,
        }
    }

    /// Minimum accepted datetime. `msg` overrides the default error (pass `""` for default).
    pub fn min(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.min_datetime = Some(Limit::new(datetime, msg));
        self
    }

    /// Maximum accepted datetime. `msg` overrides the default error (pass `""` for default).
    pub fn max(mut self, datetime: NaiveDateTime, msg: &str) -> Self {
        self.max_datetime = Some(Limit::new(datetime, msg));
        self
    }
}
field_base!(DateTimeField, label, required);

#[async_trait]
impl FormField for DateTimeField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::data::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::DateTime,
        )
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required(runique_i18n::keys::forms::error::REQUIRED) {
            return false;
        }
        let val = self.base.value.trim();

        if val.is_empty() {
            return true;
        }

        let datetime = match parse_datetime_local(val) {
            Some(dt) => dt,
            None => {
                self.set_error(t(runique_i18n::keys::forms::error::DATE_INVALID).to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = &self.min_datetime
            && datetime < min.value
        {
            let msg = min.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DATETIME_TOO_OLD,
                    &[&min.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        // Check max
        if let Some(max) = &self.max_datetime
            && datetime > max.value
        {
            let msg = max.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DATETIME_TOO_FAR,
                    &[&max.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        if let Some(dt) = parse_datetime_local(&self.base.value) {
            let mut field = self.base.clone();
            field.value = if dt.second() == 0 && dt.nanosecond() == 0 {
                dt.format("%Y-%m-%dT%H:%M").to_string()
            } else {
                // The input's default step is 60 s: a value with seconds would
                // be refused by the browser on submit unless the step allows them.
                field
                    .html_attributes
                    .entry("step".to_string())
                    .or_insert_with(|| "1".to_string());
                dt.format("%Y-%m-%dT%H:%M:%S").to_string()
            };
            context.insert("field", &field);
        }

        if let Some(min) = &self.min_datetime {
            context.insert(
                "min_datetime",
                &min.value.format("%Y-%m-%dT%H:%M").to_string(),
            );
        }
        if let Some(max) = &self.max_datetime {
            context.insert(
                "max_datetime",
                &max.value.format("%Y-%m-%dT%H:%M").to_string(),
            );
        }

        self.render_template(tera, &context)
    }
}

/// Duration input stored as seconds (`u64`). Optional min/max bounds in seconds.
#[derive(Clone, Serialize, Debug)]
pub struct DurationField {
    pub base: FieldConfig,
    pub min_seconds: Option<Limit<u64>>,
    pub max_seconds: Option<Limit<u64>>,
}

impl DurationField {
    /// Creates a duration field.
    pub fn new(name: &str) -> Self {
        Self {
            base: FieldConfig::new(name, "number", "base_datetime.html"),
            min_seconds: None,
            max_seconds: None,
        }
    }

    /// Minimum duration in seconds. `msg` overrides the default error (pass `""` for default).
    pub fn min_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.min_seconds = Some(Limit::new(seconds, msg));
        self
    }

    /// Maximum duration in seconds. `msg` overrides the default error (pass `""` for default).
    pub fn max_seconds(mut self, seconds: u64, msg: &str) -> Self {
        self.max_seconds = Some(Limit::new(seconds, msg));
        self
    }
}

field_base!(DurationField, label, required);

#[async_trait]
impl FormField for DurationField {
    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        crate::forms::data::column_value::read_field(
            &self.base,
            self.base.value.trim(),
            sea_orm::sea_query::ColumnType::String(sea_orm::sea_query::StringLen::None),
        )
    }

    async fn validate(&mut self) -> bool {
        if self.fails_required(runique_i18n::keys::forms::error::REQUIRED) {
            return false;
        }
        let val = self.base.value.trim();

        if val.is_empty() {
            return true;
        }

        // Parse the duration (in seconds)
        let seconds = match val.parse::<u64>() {
            Ok(s) => s,
            Err(_) => {
                self.set_error(t(runique_i18n::keys::forms::error::DURATION_INVALID).to_string());
                return false;
            }
        };

        // Check min
        if let Some(min) = &self.min_seconds
            && seconds < min.value
        {
            let msg = min.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DURATION_TOO_SHORT,
                    &[&min.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        // Check max
        if let Some(max) = &self.max_seconds
            && seconds > max.value
        {
            let msg = max.message_or(|| {
                tf(
                    runique_i18n::keys::forms::error::DURATION_TOO_LONG,
                    &[&max.value],
                )
            });
            self.set_error(msg);
            return false;
        }

        self.clear_error();
        true
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        let mut context = self.base_context();

        if let Some(min) = &self.min_seconds {
            context.insert("min_seconds", &min.value);
        }
        if let Some(max) = &self.max_seconds {
            context.insert("max_seconds", &max.value);
        }

        self.render_template(tera, &context)
    }
}

/// Reads a `DateTimeField` value: what the `datetime-local` input sends
/// (`YYYY-MM-DDTHH:MM[:SS]`), and what a form filled from a model holds — a
/// `timestamp` with a fractional part, or a `timestamp_tz` in RFC 3339, taken
/// in UTC like `parse_utc_datetime` stores it. The input shows neither an
/// offset nor more than seconds, so `render` rewrites the value into its shape.
pub fn parse_datetime_local(value: &str) -> Option<NaiveDateTime> {
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(value) {
        return Some(dt.naive_utc());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .or_else(|_| NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M"))
        .ok()
}

/// Reads a value meant for a `timestamp_tz` column. An RFC 3339 value keeps its
/// offset; the `YYYY-MM-DDTHH:MM[:SS]` a `datetime-local` input sends carries
/// none, so it's taken as UTC — the browser gives no way to know the user's zone.
pub fn parse_utc_datetime(value: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    parse_datetime_local(value).map(|dt| dt.and_utc())
}
