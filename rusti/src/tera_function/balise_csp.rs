// rusti/src/tera_function/nonce_balise.rs

use tera::{Result as TeraResult};
use std::collections::HashMap;
use tera::{Value};

/// Fonction Tera pour générer l'attribut nonce dans les templates
///
/// Utilisation dans les templates :
/// ```html
/// <script {{ nonce() }}>
///     console.log('Hello');
/// </script>
/// ```
///
/// Génère :
/// ```html
/// <script nonce="abc123xyz">
///     console.log('Hello');
/// </script>
/// ```
pub fn nonce_function(args: &HashMap<String, Value>) -> TeraResult<Value> {
    // Le nonce est passé dans le contexte Tera sous la clé "csp_nonce"
    if let Some(nonce_value) = args.get("csp_nonce") {
        if let Some(nonce) = nonce_value.as_str() {
            return Ok(Value::String(format!(r#"nonce="{}""#, nonce)));
        }
    }

        Ok(Value::String(String::new()))
    }
