//! `LengthConstraint` — contrainte min/max sur la longueur d'un champ texte.
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct LengthConstraint {
    pub value: u32,
    pub message: Option<String>,
}
