//! Tests — Formulaires (champs, validation, rendu, mots de passe).
//!
//! | Fichier                | Ce qui est testé                        |
//! | ---------------------- | --------------------------------------- |
//! | `test_validator`       | Règles de validation de base            |
//! | `test_prisme_rules`    | Règles Prisme (longueur, regex, …)      |
//! | `test_hidden_field`    | Champ hidden et ses variantes           |
//! | `test_special_fields`  | Champs spéciaux (fichier, datetime, …)  |
//! | `test_password`        | Hashage et vérification de mots de passe|
//! | `test_integration`     | Flux complets formulaire → validation   |

pub mod test_hidden_field;
pub mod test_integration;
pub mod test_password;
pub mod test_prisme_rules;
pub mod test_prisme_sentinel;
pub mod test_renderer;
pub mod test_special_fields;
pub mod test_validator;
