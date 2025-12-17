use tera::{Tera, Value, Result as TeraResult};
use std::collections::HashMap;

pub fn register_url(tera: &mut Tera) {
    tera.register_function("link", link_function);
}

pub fn link_function(args: &HashMap<String, Value>) -> TeraResult<Value> {
    // 1. Récupérer le nom de la route
    let link_name = args
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("link() nécessite un argument 'link'"))?;

    // 2. Cas sans paramètres
    if args.len() == 1 {
        let path = crate::macro_perso::router::reverse(link_name)
            .ok_or_else(|| tera::Error::msg(format!("Route '{}' introuvable", link_name)))?;
        return Ok(Value::String(path));
    }

    // 3. Cas avec paramètres
    let mut params: Vec<(String, String)> = Vec::new();

    for (key, value) in args.iter() {
        if key == "link" {
            continue;
        }

        let value_str = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            _ => return Err(tera::Error::msg("Paramètres invalides")),
        };

        params.push((key.clone(), value_str));
    }

    // 4. Créer les références APRÈS avoir stocké les données
    let params_refs: Vec<(&str, &str)> = params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // 5. Résoudre la route
    let path = crate::macro_perso::router::reverse_with_parameters(link_name, &params_refs)
        .ok_or_else(|| tera::Error::msg(format!("Impossible de résoudre '{}'", link_name)))?;
    Ok(Value::String(path))
}