use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

static NAME_URL: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_name_url(name: impl Into<String>, path: impl Into<String>) {
    let mut name_url_map = NAME_URL.write().unwrap();
    name_url_map.insert(name.into(), path.into());
}

pub fn reverse(name: &str) -> Option<String> {
    let name_url_map = NAME_URL.read().unwrap();
    name_url_map.get(name).cloned()
}

pub fn reverse_with_parameters(name: &str, parameters: &[(&str, &str)]) -> Option<String> {
    let path = reverse(name)?;

    Some(parameters.iter().fold(path, |acc, (key, value)| {
        acc.replace(&format!("{{{}}}", key), value)
    }))
}
