use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LengthConstraint {
    pub min: Option<usize>,
    pub max: Option<usize>,
    pub message: Option<String>,
}
