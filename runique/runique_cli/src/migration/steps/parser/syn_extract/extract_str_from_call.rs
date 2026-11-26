use super::*;

/// Searches `expr` (a method-call or function-call chain) for the first string
/// literal argument, checking the receiver before sibling arguments.
pub fn extract_str_from_call(expr: &Expr) -> Option<String> {
    match expr {
        Expr::MethodCall(mc) => {
            if let Some(s) = extract_str_from_call(&mc.receiver) {
                return Some(s);
            }
            for arg in &mc.args {
                if let Some(s) = extract_str_from_call(arg) {
                    return Some(s);
                }
            }
            None
        }
        Expr::Call(ExprCall { args, .. }) => {
            for arg in args {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = arg
                {
                    return Some(s.value());
                }
                if let Some(s) = extract_str_from_call(arg) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}
