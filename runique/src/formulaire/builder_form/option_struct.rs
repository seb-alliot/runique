use serde::Serialize;
use crate::formulaire::builder_form::base_struct::FieldConfig;

#[derive(Serialize)]

pub struct ChoiceOption {
    pub label: String,
    pub value: String,
    pub is_disabled: bool,
}

#[derive(Serialize)]

pub struct ChoiceMultiple {
    pub base: FieldConfig,
    pub options: Vec<ChoiceOption>,
    pub multiple: bool,
}

#[derive(Serialize)]

pub struct DateConfig {
    pub base: FieldConfig,
    pub min_date: Option<String>,
    pub max_date: Option<String>,
    pub format: String,
}


#[derive(Serialize)]

pub struct FileConfig {
    pub base: FieldConfig,
    pub target_path: String,
    pub max_size: usize,
    pub allowed_extensions: Vec<String>,
    pub current_file_name: Option<String>,
    pub current_file_path: Option<String>,
}

#[derive(Serialize)]

pub struct MaxlenghtConfig {
    pub max_length: usize,
}

#[derive(Serialize)]

pub struct MinlenghtConfig {
    pub min_length: usize,
}

#[derive(Serialize)]

pub struct IsRequired {
    pub choice : bool,
    pub message: Option<String>,
}


#[derive(Serialize)]
pub struct ConfigReadOnly {
    pub read: bool,
    pub reason: Option<String>,
}

#[derive(Serialize)]

pub struct ConfigDisabled {
    pub disabled: bool,
    pub reason: Option<String>,
}