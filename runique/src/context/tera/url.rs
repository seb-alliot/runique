use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use tera::{Function, Result as TeraResult, Value};

pub struct LinkFunction {
    pub url_registry: Arc<RwLock<HashMap<String, String>>>,
}

impl Function for LinkFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        link_function(args, &self.url_registry)
    }
}

fn link_function(
    args: &HashMap<String, Value>,
    url_registry: &Arc<RwLock<HashMap<String, String>>>,
) -> TeraResult<Value> {
    let link_name = args
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("link() nécessite un argument 'link'"))?;

    let map = url_registry.read().unwrap();
    let pattern = map.get(link_name).cloned().ok_or_else(|| {
        tera::Error::msg(format!(
            "Route '{}' introuvable.\n\nVérifiez que la route existe dans vos routes !",
            link_name
        ))
    })?;
    drop(map);

    Ok(Value::String(pattern))
}
