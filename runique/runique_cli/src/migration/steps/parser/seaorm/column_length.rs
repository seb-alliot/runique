use super::*;

/// The length argument of `.string_len(n)`, `.binary_len(n)` or `.var_binary(n)`.
pub(super) fn extract_length_arg(expr: &Expr) -> Option<u32> {
    collect_chain(expr).into_iter().find_map(|mc| {
        let method = mc.method.to_string();
        if !matches!(method.as_str(), "string_len" | "binary_len" | "var_binary") {
            return None;
        }
        match mc.args.first()? {
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) => n.base10_parse().ok(),
            _ => None,
        }
    })
}
