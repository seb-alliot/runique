use super::*;

/// Returns the rendered argument of the `.default(...)` call in a column chain, if any.
/// E.g. `ColumnDef::new(..).integer().default(0)` → `Some("0")`,
/// `..default(Expr::current_timestamp())` → `Some("Expr :: current_timestamp ()")`.
pub(super) fn extract_default_arg(expr: &Expr) -> Option<String> {
    for mc in collect_chain(expr) {
        if mc.method == "default"
            && let Some(arg) = mc.args.first()
        {
            return Some(arg.to_token_stream().to_string());
        }
    }
    None
}
