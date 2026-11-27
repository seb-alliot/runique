//! Tests — le moteur de templates (`runique::templates`)
//!
//! | Fichier             | Ce qui est testé                              |
//! | ------------------- | --------------------------------------------- |
//! | `test_static_tera`  | Filtres Tera : mask, csrf_field, static…      |
//! | `test_url_function` | LinkFunction : résolution d'URLs nommées      |
//! | `test_autoescape`   | Contrat d'échappement : ce qui sort en HTML   |

pub mod test_autoescape;
pub mod test_static_tera;
pub mod test_url_function;
