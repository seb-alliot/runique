//! Reads the `extend!{}` blocks of a Rust source file — the fields an app adds
//! to a framework table. Parsed by `runique_dsl`, like the `extend!{}` macro.
//!
//! ```rust,ignore
//! extend! {
//!     table: "eihwaz_users",
//!     fields: {
//!         avatar: image [upload_to: "avatars/"],
//!         bio: textarea,
//!         website: url [required],
//!     }
//! }
//! ```
use anyhow::Result;
use runique_dsl::extend::ExtendDsl;
use syn::visit::Visit;

use crate::migration::utils::parser_builder::{MacroCollector, decl_to_column, located};
use crate::migration::utils::types::ParsedSchema;

/// Parses all `extend!{}` blocks in a Rust source file, one [`ParsedSchema`]
/// per block, with `primary_key = None` — these extend existing tables. An
/// error when the file isn't valid Rust or a block doesn't parse.
pub fn parse_extend_blocks_from_source(source: &str) -> Result<Vec<ParsedSchema>> {
    let file = syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!(located(&e)))?;
    let mut collector = MacroCollector::<ExtendDsl>::new("extend");
    collector.visit_file(&file);

    let mut schemas = Vec::new();
    for parsed in collector.found {
        let ext = parsed.map_err(|e| anyhow::anyhow!(located(&e)))?;
        schemas.push(ParsedSchema {
            columns: ext
                .fields
                .iter()
                .map(|decl| decl_to_column(decl, &ext.enums))
                .collect(),
            table_name: ext.table,
            primary_key: None,
            foreign_keys: Vec::new(),
            indexes: Vec::new(),
        });
    }
    Ok(schemas)
}
