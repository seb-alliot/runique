use super::*;

/// Extracts (enum_name, string_values) from an expression `.enum_type("Name", vec![...])`
pub(super) fn extract_enum_type_info(expr: &Expr) -> (Option<String>, Vec<String>) {
    let chain = collect_chain(expr);
    for mc in &chain {
        if mc.method != "enum_type" {
            continue;
        }
        // First arg: enum name (string literal)
        let enum_name = mc.args.first().and_then(|a| {
            if let Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(s),
                ..
            }) = a
            {
                Some(s.value())
            } else {
                None
            }
        });
        // Second arg: vec![...] of strings
        let values = if mc.args.len() >= 2 {
            extract_all_str_args(&mc.args[1])
        } else {
            Vec::new()
        };
        return (enum_name, values);
    }
    (None, Vec::new())
}
