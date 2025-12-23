use crate::formulaire::field::RustiField;
use std::collections::HashMap;
use serde_json::Value;
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Forms {
    #[serde(default)]
    pub errors: HashMap<String, String>,

    #[serde(default)]
    pub cleaned_data: HashMap<String, Value>,
}

impl Forms {
    pub fn new() -> Self {
        Self {
            errors: HashMap::new(),
            cleaned_data: HashMap::new(),
        }
    }

    pub fn field<F: RustiField>(
        &mut self,
        name: &str,
        field: &F,
        raw_value: &str
    ) -> Option<F::Output>
    where F::Output: Serialize + Clone
    {
        match field.process(raw_value) {
            Ok(value) => {
                if let Ok(json_val) = serde_json::to_value(value.clone()) {
                    self.cleaned_data.insert(name.to_string(), json_val);
                }
                Some(value)
            },
            Err(e) => {
                self.errors.insert(name.to_string(), e);
                None
            }
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn is_not_valid(&self) -> bool {
        !self.is_valid()
    }

    pub fn get_value<T: DeserializeOwned + 'static + Clone + Send + Sync>(&self, field_name: &str) -> Option<T> {
        self.cleaned_data.get(field_name).and_then(|value| {
            serde_json::from_value(value.clone()).ok()
        })
    }
}

pub trait FormulaireTrait: Send + Sync + 'static {
    fn new() -> Self;
    fn validate(&mut self, raw_data: &HashMap<String, String>) -> bool;
}


