use crate::forms::base::FieldConfig;
use serde::Serialize;

// #[derive(Serialize, Clone )]
// pub struct ChoiceOption {
//     pub label: String,
//     pub value: String,
//     pub is_disabled: bool,
// }

// #[derive(Serialize, Clone )]
// pub struct ChoiceMultiple {
//     pub base: FieldConfig,
//     pub options: Vec<ChoiceOption>,
//     pub multiple: bool,
// }

#[derive(Serialize, Clone)]
pub struct DateConfig {
    pub base: FieldConfig,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub format: String,
}

#[derive(Serialize, Clone)]
pub struct FileConfig {
    pub base: FieldConfig,
    pub target_path: String,
    pub max_size: usize,
    pub allowed_extensions: Vec<String>,
    pub current_file_name: Option<String>,
    pub current_file_path: Option<String>,
}
