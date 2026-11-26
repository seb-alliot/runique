use super::*;

/// Same as [`extract_alias_new_str`], but only matches when `expr` itself is
/// directly the `Alias::new(...)` call — it does not recurse into a receiver
/// or nested arguments.
pub fn extract_alias_new_str_inner(expr: &Expr) -> Option<String> {
    if let Expr::Call(ExprCall { func, args, .. }) = expr {
        let is_alias = if let Expr::Path(p) = func.as_ref() {
            p.path.segments.iter().any(|s| s.ident == "Alias")
        } else {
            false
        };
        if is_alias
            && let Some(Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            })) = args.first()
        {
            return Some(s.value());
        }
    }
    None
}
