//! Reads the `model!{}` DSL out of a Rust source file into a [`ParsedSchema`].
//! The DSL itself is parsed by `runique_dsl`, the same parser the `model!{}`
//! macro uses, so a model the macro refuses is refused here too.
mod to_schema;

use anyhow::{Result, bail};
use runique_dsl::ast::ModelInput;
use syn::visit::Visit;

pub(crate) use to_schema::decl_to_column;
use to_schema::{model_to_parsed_schema, pascal_to_snake};

use crate::migration::utils::types::ParsedSchema;

/// `line:column: message` — where a DSL error is in the scanned file.
pub(crate) fn located(err: &syn::Error) -> String {
    let start = err.span().start();
    format!("{}:{}: {}", start.line, start.column + 1, err)
}

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

/// Parses the `model!{}` invocation of a Rust source file and returns the
/// model's snake_case name together with its [`ParsedSchema`]. `Ok(None)` when
/// the file has no `model!{}`; an error when the file isn't valid Rust, when
/// the model doesn't parse, or when the file declares more than one model.
pub fn parse_schema_from_source(source: &str) -> Result<Option<(String, ParsedSchema)>> {
    let file = syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!(located(&e)))?;
    let mut collector = MacroCollector::<ModelInput>::new("model");
    collector.visit_file(&file);

    let mut models = Vec::new();
    for parsed in collector.found {
        models.push(parsed.map_err(|e| anyhow::anyhow!(located(&e)))?);
    }
    if models.len() > 1 {
        bail!(
            "{} `model!{{}}` declarations in one file — one model per file",
            models.len()
        );
    }
    Ok(models.pop().map(|model| {
        (
            pascal_to_snake(&model.name.to_string()),
            model_to_parsed_schema(&model),
        )
    }))
}
