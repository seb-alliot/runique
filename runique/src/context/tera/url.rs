//! Tera `link` function — named URL resolution from the global registry, with optional parameters.
use crate::utils::aliases::{ARlockmap, JsonMap};
use serde_json::Value;
use tera::{Function, Kwargs, State, TeraResult};

pub struct LinkFunction {
    pub url_registry: ARlockmap,
}

impl Function<TeraResult<String>> for LinkFunction {
    fn call(&self, kwargs: Kwargs, _: &State) -> TeraResult<String> {
        link_function(&kwargs, &self.url_registry)
    }
}

fn link_function(kwargs: &Kwargs, url_registry: &ARlockmap) -> TeraResult<String> {
    // A missing or non-string `link` is reported by Tera itself.
    let link_name = kwargs.must_get::<&str>("link")?;

    let map = url_registry.read().unwrap_or_else(|e| e.into_inner());
    let pattern = map.get(link_name).cloned().ok_or_else(|| {
        tera::Error::message(format!(
            "Route '{}' not found.\n\nVerify that the route exists in your routes!",
            link_name
        ))
    })?;
    drop(map);

    // The remaining arguments are route/query parameters of arbitrary shape
    // (`query` may itself be an object). Deserializing into `serde_json::Value`
    // keeps the pattern-matching below on a public enum — `tera::Value` has
    // private fields and no variants to match on.
    let args: JsonMap = kwargs.deserialize()?;

    // Substitute route parameters {id}, {slug}, etc.
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

    // Handle query string parameters
    if let Some(query_val) = args.get("query") {
        let query_str = match query_val {
            Value::String(s) => s.clone(),
            Value::Object(map) => {
                // Build ?k1=v1&k2=v2
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

    Ok(result)
}

// Usage examples in templates:
//
// Simple route:
//   {% link "index" %}
//   → /
//
// Route parameter:
//   {% link "article_detail" id=article.id %}
//   → /articles/42
//
// Without query:
//   {% link "article_list" %}
//   → /articles
//
// With query (object):
//   {% link "article_list" query={page: 2, search: "rust"} %}
//   → /articles?page=2&search=rust
//
// With query (raw string):
//   {% link "article_list" query="page=2&search=rust" %}
//   → /articles?page=2&search=rust
