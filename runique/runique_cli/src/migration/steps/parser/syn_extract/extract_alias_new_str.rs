use super::*;

/// Searches `expr` for a call shaped like `sea_query::Alias::new("name")` (or
/// `Alias::new("name")`) anywhere in a method/function-call chain and returns
/// the literal `"name"`.
pub fn extract_alias_new_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::MethodCall(mc) => {
            if let Some(s) = extract_alias_new_str(&mc.receiver) {
                return Some(s);
            }
            for arg in &mc.args {
                if let Some(s) = extract_alias_new_str(arg) {
                    return Some(s);
                }
            }
            None
        }
        Expr::Call(ExprCall { func, args, .. }) => {
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

            for arg in args {
                if let Some(s) = extract_alias_new_str(arg) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}
