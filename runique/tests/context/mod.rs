//! Tests — Context (Tera fonctions, filtres, URL)
//!
//! | Fichier                  | Ce qui est testé                          |
//! | ------------------------ | ----------------------------------------- |
//! | `test_csp_function`      | nonce_function : nonce CSP pour templates |
//! | `test_static_tera`       | Filtres Tera : mask, csrf_field, static… |
//! | `test_url_function`      | LinkFunction : résolution d'URLs nommées  |

pub mod test_app_error;
pub mod test_request_extensions;
pub mod test_runique_context;
pub mod test_static_tera;
pub mod test_template_request;
pub mod test_url_function;
