//! AST Parser for the `model!{}` DSL — extracts a [`crate::model::ast::ModelInput`]
//! from the macro's token stream. Split by concern: one file per `impl Parse`
//! (`model_input`, `enum_def`, `pk`, `fk`, `relation`, `meta`), and the
//! `form_field/` submodule for the field-declaration grammar (parsing,
//! attribute validation, and conversion to the SQL-level `FieldDef`).
mod enum_def;
mod fk;
mod form_field;
mod meta;
mod model_input;
mod pk;
mod relation;
mod sql_identifier;
