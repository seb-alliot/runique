//! Fonction Tera `link` — résolution d'URL nommée depuis le registre global, avec paramètres optionnels.
use crate::utils::aliases::{ARlockmap, JsonMap, TResult};
use tera::{Function, Value};

pub struct LinkFunction {
    pub url_registry: ARlockmap,
}

impl Function for LinkFunction {
    fn call(&self, args: &JsonMap) -> TResult {
        link_function(args, &self.url_registry)
    }
}

fn link_function(args: &JsonMap, url_registry: &ARlockmap) -> TResult {
    let link_name = args
        .get("link")
        .and_then(|v| v.as_str())
        .ok_or_else(|| tera::Error::msg("link() nécessite un argument 'link'"))?;

    let map = url_registry.read().unwrap_or_else(|e| e.into_inner());
    let pattern = map.get(link_name).cloned().ok_or_else(|| {
        tera::Error::msg(format!(
            "Route '{}' introuvable.\n\nVérifiez que la route existe dans vos routes !",
            link_name
        ))
    })?;
    drop(map);

    // Substituer les paramètres de route {id}, {slug}, etc.
    let mut result = args
        .iter()
        .filter(|(k, _)| *k != "link" && *k != "query")
        .fold(pattern, |acc, (k, v)| {
            let value = match v {
                Value::String(s) => s.clone(),
                Value::Number(n) => n.to_string(),
                _ => v.to_string(),
            };
            acc.replace(&format!("{{{}}}", k), &value)
        });

    // Gérer les paramètres de query string
    if let Some(query_val) = args.get("query") {
        let query_str = match query_val {
            Value::String(s) => s.clone(),
            Value::Object(map) => {
                // Construire ?k1=v1&k2=v2
                map.iter()
                    .map(|(k, v)| {
                        let v_encoded: String = match v {
                            Value::String(s) => urlencoding::encode(s).into_owned(),
                            Value::Number(n) => n.to_string(),
                            _ => urlencoding::encode(&v.to_string()).into_owned(),
                        };
                        format!("{}={}", k, v_encoded)
                    })
                    .collect::<Vec<_>>()
                    .join("&")
            }
            _ => query_val.to_string(),
        };

        if !query_str.is_empty() {
            result.push('?');
            result.push_str(&query_str);
        }
    }

    Ok(Value::String(result))
}

// Exemples d'utilisation dans les templates :
//
// Route simple :
//   {% link "index" %}
//   → /
//
// Paramètre de route :
//   {% link "article_detail" id=article.id %}
//   → /articles/42
//
// Sans query :
//   {% link "article_list" %}
//   → /articles
//
// Avec query (objet) :
//   {% link "article_list" query={page: 2, search: "rust"} %}
//   → /articles?page=2&search=rust
//
// Avec query (string brute) :
//   {% link "article_list" query="page=2&search=rust" %}
//   → /articles?page=2&search=rust
