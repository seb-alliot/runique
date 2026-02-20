use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LengthConstraint {
    pub value: u32,
    pub message: Option<String>,
}
