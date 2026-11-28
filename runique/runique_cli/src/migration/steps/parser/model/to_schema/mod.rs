//! Conversion from the `runique_dsl` AST to the engine-agnostic
//! [`ParsedSchema`] the migration generator consumes.
mod column;
mod column_type;
mod enum_values;
mod find_enum;
mod list_table;
mod table;

use quote::ToTokens;
use runique_dsl::{
    ast::{
        EnumBackingType, EnumDef, FieldOption, FieldType, FkAction, FormFieldAttr, FormFieldDecl,
        ModelInput, RelationDef,
    },
    form_field_to_field_def,
};

use crate::migration::types::{ColumnType, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};
use runique_dsl::snake_case;

pub(crate) use column::decl_to_column;
use column_type::col_type;
use enum_values::enum_values;
use find_enum::find_enum;
pub(crate) use list_table::list_tables;
pub(crate) use table::model_to_parsed_schema;
