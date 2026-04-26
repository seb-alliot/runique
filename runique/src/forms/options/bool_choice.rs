//! `BoolChoice` — validation option for boolean fields (expected value true/false/any).
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct BoolChoice {
    pub choice: bool,
    pub message: Option<String>,
}

impl BoolChoice {
    pub fn new(choice: bool, message: Option<String>) -> Self {
        Self { choice, message }
    }
}
