//! Reads the `model!{}` DSL out of a Rust source file into a [`ParsedSchema`].
//! The DSL itself is parsed by `runique_dsl`, the same parser the `model!{}`
//! macro uses, so a model the macro refuses is refused here too.
mod declared_renames;
mod located;
mod parse_model_from_source;
mod parse_schema_from_source;

mod to_schema;

use anyhow::{Result, bail};
use runique_dsl::ast::{FormFieldAttr, ModelInput};
use syn::visit::Visit;

pub(crate) use to_schema::decl_to_column;
use to_schema::{list_tables, model_to_parsed_schema};

use crate::migration::types::ParsedSchema;
use runique_dsl::snake_case;

/// Collects every invocation of the macro named `name` in a file, parsed
/// with `T`'s grammar.
pub(crate) struct MacroCollector<T> {
    name: &'static str,
    pub found: Vec<syn::Result<T>>,
}

impl<T> MacroCollector<T> {
    pub(crate) fn new(name: &'static str) -> Self {
        Self {
            name,
            found: Vec::new(),
        }
    }
}

impl<'ast, T: syn::parse::Parse> Visit<'ast> for MacroCollector<T> {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if mac
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == self.name)
        {
            self.found.push(syn::parse2::<T>(mac.tokens.clone()));
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// A `model!{}` read for migrations: its table, and the tables of its list
/// fields (`checkbox` / `multichoice`).
pub struct ParsedModel {
    /// The model's snake_case name.
    pub name: String,
    pub schema: ParsedSchema,
    pub lists: Vec<ParsedSchema>,
    pub managed: bool,
    /// What `renamed_from` declares, as (former table, table): the model's
    /// own table, and the table of each list field, which is named after
    /// both and so follows either rename.
    pub renames: Vec<(String, String)>,
}

use declared_renames::declared_renames;
pub(crate) use located::located;
pub use parse_model_from_source::parse_model_from_source;
pub use parse_schema_from_source::parse_schema_from_source;
