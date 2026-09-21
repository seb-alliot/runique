//! `LengthConstraint` — min/max constraint on the length of a text field.
use serde::{Deserialize, Serialize};

/// A min or max length bound (character count) with an optional custom error
/// message shown when the constraint is violated.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LengthConstraint {
    pub value: u32,
    pub message: Option<String>,
}
