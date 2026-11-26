use super::*;

pub(super) fn extract_seaorm_index(expr: &Expr) -> Option<ParsedIndex> {
    let chain = collect_chain(expr);
    let mut name: Option<String> = None;
    let mut columns = Vec::new();
    let mut unique = false;

    for mc in &chain {
        match mc.method.to_string().as_str() {
            "name" => {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = mc.args.first()
                {
                    name = Some(s.value());
                }
            }
            "col" => {
                if let Some(arg) = mc.args.first()
                    && let Some(col) =
                        extract_alias_new_str_inner(arg).or_else(|| extract_str_from_call(arg))
                {
                    columns.push(col);
                }
            }
            "unique" => {
                unique = true;
            }
            _ => {}
        }
    }

    Some(ParsedIndex {
        name: name?,
        columns,
        unique,
    })
}
