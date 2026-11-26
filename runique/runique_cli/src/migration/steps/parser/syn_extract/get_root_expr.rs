use super::*;

/// Walks down a method-call chain and returns its receiver at the root
/// (e.g. `ColumnDef::new("x")` in `ColumnDef::new("x").integer().unique()`).
pub fn get_root_expr(expr: &Expr) -> &Expr {
    let mut current = expr;
    loop {
        if let Expr::MethodCall(mc) = current {
            current = &mc.receiver;
        } else {
            return current;
        }
    }
}
