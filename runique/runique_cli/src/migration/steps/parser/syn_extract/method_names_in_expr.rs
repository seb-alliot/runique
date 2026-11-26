use super::*;

/// Collects every method and function-call name reachable from `expr`, recursing
/// through receivers and arguments — used to detect which builder methods were
/// called anywhere in a column definition, regardless of call order.
pub fn method_names_in_expr(expr: &Expr) -> Vec<String> {
    let mut names = Vec::new();
    collect_method_names(expr, &mut names);
    names
}
