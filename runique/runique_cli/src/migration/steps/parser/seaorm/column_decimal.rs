use super::*;

/// `.decimal_len(p, s)` in a snapshot's column chain: the decimal's precision and scale.
pub(super) fn extract_decimal_len_arg(expr: &Expr) -> Option<(u32, u32)> {
    collect_chain(expr).into_iter().find_map(|mc| {
        if mc.method != "decimal_len" {
            return None;
        }
        let int = |arg: Option<&Expr>| match arg? {
            Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Int(n),
                ..
            }) => n.base10_parse().ok(),
            _ => None,
        };
        Some((int(mc.args.first())?, int(mc.args.iter().nth(1))?))
    })
}
