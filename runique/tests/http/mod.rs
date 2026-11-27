//! Tests — ce que le handler reçoit (`runique::http`)
//!
//! | Fichier                   | Ce qui est testé                                   |
//! | ------------------------- | -------------------------------------------------- |
//! | `test_template_request`   | `Request` : extraction, rendu, formulaires         |
//! | `test_runique_context`    | `RuniqueContext` : moteur, session, CSRF, flash    |
//! | `test_request_extensions` | `RequestExtensions` : ce qui est injecté par requête |

pub mod test_request_extensions;
pub mod test_runique_context;
pub mod test_template_request;
