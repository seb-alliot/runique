use super::*;

/// Collects every string literal reachable from `expr`, including inside
/// `vec![...]` macro invocations (e.g. a list of enum variant names).
pub fn extract_all_str_args(expr: &Expr) -> Vec<String> {
    let mut result = Vec::new();
    collect_str_args(expr, &mut result);
    result
}
