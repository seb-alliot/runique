use super::*;

/// Finds a `.references(...)` call anywhere in `expr` and returns
/// `(to_table, to_column)`, defaulting the column to `"id"` when only the
/// table is given.
pub fn extract_references_from_expr(expr: &Expr) -> Option<(String, String)> {
    if let Expr::MethodCall(mc) = expr {
        if mc.method == "references" {
            let strings = extract_all_str_args(&mc.args[0]);
            return match strings.len() {
                0 => None,
                1 => Some((strings[0].clone(), "id".to_string())),
                _ => Some((strings[0].clone(), strings[1].clone())),
            };
        }
        if let Some(s) = extract_references_from_expr(&mc.receiver) {
            return Some(s);
        }
        for arg in &mc.args {
            if let Some(s) = extract_references_from_expr(arg) {
                return Some(s);
            }
        }
    }
    None
}
