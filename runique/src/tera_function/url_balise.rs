use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tera::{Function, Result as TeraResult, Value};

pub struct LinkFunction {
    url_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl Function for LinkFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        link_function(args, &self.url_registry)
    }
}

pub fn register_link_function(
    tera: &mut tera::Tera,
    url_registry: Arc<RwLock<HashMap<String, String>>>,
) {
    tera.register_function("link", LinkFunction { url_registry });
}

fn link_function(
    args: &HashMap<String, Value>,
    url_registry: &Arc<RwLock<HashMap<String, String>>>,
) -> TeraResult<Value> {
    // Ton code existant, mais en utilisant url_registry au lieu de state
    let link_name = args
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("link() nécessite un argument 'link'"))?;

    let map = url_registry.read().unwrap();
    let pattern = map.get(link_name).cloned().ok_or_else(|| {
        tera::Error::msg(format!(
            "Route '{}' introuvable.\n\nVérifiez que la route existe dans votre urlpatterns!",
            link_name
        ))
    })?;
    drop(map);

    // ... ton code existant pour extract_placeholders, etc ...

    Ok(Value::String(pattern)) // simplifié pour l'exemple
}
