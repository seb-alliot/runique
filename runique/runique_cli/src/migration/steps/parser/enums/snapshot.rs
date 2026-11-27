use super::*;

/// Finds the type an enum snapshot creates: the name given to `as_enum`, the
/// values given to `values`.
#[derive(Default)]
struct TypeCreated {
    name: Option<String>,
    values: Option<Vec<String>>,
}

impl<'ast> Visit<'ast> for TypeCreated {
    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if call.method == "as_enum" && self.name.is_none() {
            self.name = call.args.first().and_then(extract_alias_new_str);
        }
        if call.method == "values"
            && self.values.is_none()
            && let Some(syn::Expr::Array(list)) = call.args.first()
        {
            self.values = list.elems.iter().map(extract_alias_new_str).collect();
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

/// Reads an enum snapshot back: its name and its values, in the order of the
/// file. Refused when the file creates no enum type, or when one of its
/// values is not written as a name.
pub fn parse_enum_snapshot(source: &str) -> Result<ParsedEnum> {
    let file =
        syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    let mut found = TypeCreated::default();
    found.visit_file(&file);
    match (found.name, found.values) {
        (Some(name), Some(values)) => Ok(ParsedEnum {
            name,
            values,
            places: Vec::new(),
        }),
        _ => anyhow::bail!("Cannot extract enum type"),
    }
}
