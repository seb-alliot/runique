use super::*;

/// True if `expr`'s method-chain root is `ColumnDef::new(...)` — the shape of a real
/// column definition inside `Table::create()...col(...)`. `Index::create()...col(Alias::new(name))`
/// (a composite index referencing an existing column by name) also has method name `col`,
/// but its argument is a bare `Alias::new(...)` with no `ColumnDef` wrapper — without this
/// guard, that index reference was mistaken for a second, type-less column definition,
/// which silently overwrote the real one (defaulting to `col_type: "String"`) whenever a
/// column happened to also appear in a `unique_together`/`indexes:` group.
pub(super) fn is_column_def(expr: &Expr) -> bool {
    if let Expr::Call(syn::ExprCall { func, .. }) = get_root_expr(expr)
        && let Expr::Path(p) = func.as_ref()
    {
        return p.path.segments.iter().any(|s| s.ident == "ColumnDef");
    }
    false
}
