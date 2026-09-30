//! Reads the Runique DSL — `model!{}` and `extend!{}` — into an AST, without
//! generating any code. The `derive_form` macros and the `makemigrations` CLI
//! both read the DSL through this crate, so they can't disagree on it.
pub mod ast;
pub mod extend;
mod parser;
pub mod types;

pub use parser::form_field::form_field_to_field_def;
