use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct LengthConstraint {
    pub limit: usize,
    pub message: Option<String>,
}

impl LengthConstraint {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            message: None,
        }
    }
}
