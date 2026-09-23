//! AST Parser for the builder DSL (`ModelSchema`) — extracts the `ParsedSchema` from Rust source code.
//! Supports two syntaxes:
//!   v1: `fields: { name: String [required], ... }`
//!   v2: `{ name: text [required], ... }` (anonymous block, semantic types)
mod field;
mod model;
mod relation;
mod to_schema;
mod type_mapping;

use model::DslModel;
use syn::visit::Visit;
use to_schema::{dsl_to_parsed_schema, pascal_to_snake};

use crate::migration::utils::types::ParsedSchema;

struct DslVisitor {
    pub schema: Option<ParsedSchema>,
    pub model_name: Option<String>,
}

impl DslVisitor {
    fn new() -> Self {
        Self {
            schema: None,
            model_name: None,
        }
    }
}

impl<'ast> Visit<'ast> for DslVisitor {
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if self.schema.is_some() {
            return;
        }
        let is_model = mac
            .path
            .segments
            .last()
            .map(|s| s.ident == "model")
            .unwrap_or(false);

        if is_model && let Ok(model) = syn::parse2::<DslModel>(mac.tokens.clone()) {
            self.model_name = Some(pascal_to_snake(&model.name));
            self.schema = Some(dsl_to_parsed_schema(model));
        }
        syn::visit::visit_macro(self, mac);
    }
}

/// Parses a `model!{}` DSL invocation out of a Rust source file and returns
/// the model's snake_case name together with its [`ParsedSchema`]. Supports
/// both the legacy v1 syntax (`fields: { name: String [required], ... }`) and
/// the v2 semantic-type syntax (`{ name: text [required], ... }`). Returns
/// `None` if the source doesn't parse or contains no `model!{}` call.
pub fn parse_schema_from_source(source: &str) -> Option<(String, ParsedSchema)> {
    let file = syn::parse_str::<syn::File>(source).ok()?;
    let mut visitor = DslVisitor::new();
    visitor.visit_file(&file);
    let schema = visitor.schema?;
    let model_name = visitor.model_name.unwrap_or_default();
    Some((model_name, schema))
}
