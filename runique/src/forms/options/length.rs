use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LengthConstraint<T> {
    pub value: T,
    pub message: Option<String>,
}
