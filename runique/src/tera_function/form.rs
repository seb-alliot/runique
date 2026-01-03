use std::collections::HashMap;
use tera::{Result as TeraResult, Tera, Value};

pub fn register_form_filter(tera: &mut Tera) {
    tera.register_filter("form_html", form_html_filter);
}

fn form_html_filter(value: &Value, _args: &HashMap<String, Value>) -> TeraResult<Value> {
    // Extraire le champ "html" de l'objet sérialisé
    if let Some(obj) = value.as_object() {
        if let Some(html) = obj.get("html") {
            return Ok(html.clone());
        }
    }

    // Si pas trouvé, retourner une erreur claire
    Err(tera::Error::msg("Le formulaire n'a pas de champ 'html'. Assurez-vous que Forms implémente Serialize correctement."))
}
