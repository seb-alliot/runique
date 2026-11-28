//! `Limit`: a bound a field checks, and the message shown when it's crossed.
use serde::{Deserialize, Serialize};

/// A bound (a length, a date, a duration…) with an optional custom error
/// message shown when a value crosses it.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Limit<T> {
    pub value: T,
    pub message: Option<String>,
}

impl<T> Limit<T> {
    /// An empty `message` keeps the field's default error.
    pub fn new(value: T, message: &str) -> Self {
        Self {
            value,
            message: (!message.is_empty()).then(|| message.to_string()),
        }
    }

    /// The custom message, else `default`.
    pub fn message_or(&self, default: impl FnOnce() -> String) -> String {
        self.message.clone().unwrap_or_else(default)
    }
}
