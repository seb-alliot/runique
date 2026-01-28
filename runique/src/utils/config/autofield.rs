use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoFieldType {
    AutoField,    // correspond à i32
    BigAutoField, // correspond à i64
}

impl Default for AutoFieldType {
    fn default() -> Self {
        AutoFieldType::AutoField
    }
}

impl AutoFieldType {
    pub fn from_str(field_str: &str) -> Self {
        match field_str {
            "runique.db.models.BigAutoField" => AutoFieldType::BigAutoField,
            _ => AutoFieldType::AutoField,
        }
    }

    /// Retourne la taille en bits (ou type Rust équivalent)
    pub fn rust_type(&self) -> &'static str {
        match self {
            AutoFieldType::AutoField => "i32",
            AutoFieldType::BigAutoField => "i64",
        }
    }
    pub fn from_env() -> Self {
        std::env::var("DEFAULT_AUTO_FIELD")
            .map(|s| Self::from_str(&s))
            .unwrap_or_default()
    }
}
