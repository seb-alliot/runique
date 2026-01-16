use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct BoolChoice {
    pub choice: bool,
    pub message: Option<String>,
}

impl BoolChoice {
    pub fn new(choice: bool, message: Option<String>) -> Self {
        Self { choice, message }
    }
}
