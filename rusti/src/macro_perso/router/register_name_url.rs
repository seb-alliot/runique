use std::collections::HashMap;
use std::sync::RwLock;
use once_cell::sync::Lazy;

// A global HashMap to store name to URL mappings
static NAME_URL: Lazy<RwLock<HashMap<String, String>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

// Function to register a name-URL pair
pub fn register_name_url(name: impl Into<String>, path: impl Into<String>) {
    let mut name_url_map = NAME_URL.write().unwrap();
    name_url_map.insert(name.into(), path.into());
}

// Function to retrieve a URL by its registered name
pub fn reverse(name: &str) -> Option<String> {
    let name_url_map = NAME_URL.read().unwrap();
    name_url_map.get(name).cloned()
}

pub fn reverse_with_parameters(name: &str, parameters: &[(&str, &str)]) -> Option<String> {

    let mut path = reverse(name);
    for (key, value) in parameters {
        let placeholder = format!("{{{}}}", key);
        path = Some(path?.replace(&placeholder, value));
    }
    path
}