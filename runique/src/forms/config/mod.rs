//! A field's configuration: `FieldConfig`, and the rules it carries with
//! their message (`Limit`, `BoolChoice`).
pub mod bool_choice;
pub mod field_config;
pub mod limit;

pub use bool_choice::*;
pub use field_config::*;
pub use limit::*;
