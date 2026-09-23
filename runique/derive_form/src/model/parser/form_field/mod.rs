//! `{ name: type [attrs], ... }` field declaration — parsing, cross-attribute
//! validation, and conversion to the SQL-level `FieldDef`.
pub(crate) mod attr_name;
pub(crate) mod parse;
pub(crate) mod size;
pub(crate) mod suggest_type;
pub(crate) mod to_field_def;
pub(crate) mod validate_attrs;

pub(crate) use to_field_def::form_field_to_field_def;
