use crate::app_state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use tera::{Function, Result as TeraResult, Tera, Value};

/// Enregistre la fonction `link()` dans Tera
pub struct LinkFunction {
    state: Arc<AppState>,
}

impl Function for LinkFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        link_function(args, &self.state)
    }
}

pub fn register_url(tera: &mut Tera, state: Arc<AppState>) {
    tera.register_function("link", LinkFunction { state });
}

/// Extrait les placeholders d'une URL
/// Exemple : "/user/{id}/post/{slug}" → ["id", "slug"]
fn extract_placeholders(path: &str) -> Vec<String> {
    let mut placeholders = Vec::new();
    let mut current = String::new();
    let mut in_placeholder = false;

    for c in path.chars() {
        match c {
            '{' => in_placeholder = true,
            '}' => {
                if !current.is_empty() {
                    placeholders.push(current.clone());
                    current.clear();
                }
                in_placeholder = false;
            }
            _ if in_placeholder => current.push(c),
            _ => {}
        }
    }
    placeholders
}

pub fn link_function(args: &HashMap<String, Value>, state: &AppState) -> TeraResult<Value> {
    // 1. Récupérer le nom de la route
    let link_name = args
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("link() nécessite un argument 'link'"))?;

    // 2. Résoudre le pattern de la route
    let map = state.url_registry.read().unwrap();
    let pattern = map.get(link_name).cloned().ok_or_else(|| {
        tera::Error::msg(format!(
            "Route '{}' introuvable.\n\nVérifiez que la route existe dans votre urlpatterns!",
            link_name
        ))
    })?;

    // 3. Extraire les placeholders attendus
    let expected_params = extract_placeholders(&pattern);

    // 4. Cas sans paramètres
    if expected_params.is_empty() {
        if args.len() > 1 {
            return Err(tera::Error::msg(format!(
                "La route '{}' ne prend pas de paramètres, mais vous avez fourni : {:?}",
                link_name,
                args.keys().filter(|k| *k != "link").collect::<Vec<_>>()
            )));
        }
        return Ok(Value::String(pattern));
    }

    // 5. Collecter les paramètres fournis
    let mut list_params: Vec<(String, String)> = Vec::new();

    for (key, value) in args.iter() {
        if key == "link" {
            continue;
        }

        let value_str = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => {
                return Err(tera::Error::msg(format!(
                    "Le paramètre '{}' a un type invalide. Utilisez String, Number ou Bool.",
                    key
                )));
            }
        };

        list_params.push((key.clone(), value_str));
    }

    // 6. Vérifier les paramètres manquants
    let list_keys: Vec<&str> = list_params.iter().map(|(k, _)| k.as_str()).collect();

    let missing_params: Vec<&String> = expected_params
        .iter()
        .filter(|p| !list_keys.contains(&p.as_str()))
        .collect();

    if !missing_params.is_empty() {
        let mut message = format!("Paramètres manquants pour la route '{}':\n\n", link_name);
        message.push_str(&format!("Pattern: {}\n\n", pattern));
        message.push_str("Paramètres requis:\n");
        for p in &expected_params {
            if missing_params.contains(&p) {
                message.push_str(&format!("{} (manquant)\n", p));
            } else {
                message.push_str(&format!("{} (fourni)\n", p));
            }
        }
        message.push_str(&format!(
            "\nExemple: {{{{ link(link='{}', {}) }}}}\n",
            link_name,
            expected_params
                .iter()
                .map(|p| format!("{}=value", p))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        return Err(tera::Error::msg(message));
    }

    // 7. Vérifier les paramètres en trop
    let extra_params: Vec<&String> = list_params
        .iter()
        .map(|(k, _)| k)
        .filter(|k| !expected_params.contains(k))
        .collect();

    if !extra_params.is_empty() {
        return Err(tera::Error::msg(format!(
            "Paramètres non reconnus pour la route '{}':\n\n\
            Pattern: {}\n\
            Paramètres attendus: {:?}\n\
            Paramètres en trop: {:?}",
            link_name, pattern, expected_params, extra_params
        )));
    }

    // 8. Créer les références
    let list_refs: Vec<(&str, &str)> = list_params
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    // 9. Résoudre l'URL finale
    let final_url = {
        let mut url = pattern.clone();
        for (k, v) in list_refs {
            url = url.replace(&format!("{{{}}}", k), v);
        }
        url
    };
    Ok(Value::String(final_url))
}
