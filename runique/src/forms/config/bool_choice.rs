//! `BoolChoice` — validation option for boolean fields (expected value true/false/any).
use serde::{Deserialize, Serialize};

/// A boolean validation flag (e.g. "is this field required?") paired with an
/// optional custom error message shown when the check fails.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BoolChoice {
    pub choice: bool,
    pub message: Option<String>,
}

impl BoolChoice {
    /// Creates a flag with the given `choice` and an optional message
    /// overriding the default error text.
    pub fn new(choice: bool, message: Option<String>) -> Self {
        Self { choice, message }
    }
}
