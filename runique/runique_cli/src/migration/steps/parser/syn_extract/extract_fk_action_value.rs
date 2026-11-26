use super::*;

/// Maps a path expression (e.g. `ForeignKeyAction::Cascade`) to its action.
/// Anything unrecognized, including a bare path with no last segment, resolves to `NoAction`.
pub fn extract_fk_action_value(expr: &Expr) -> FkAction {
    if let Expr::Path(p) = expr
        && let Some(seg) = p.path.segments.last()
    {
        return match seg.ident.to_string().as_str() {
            "Cascade" => FkAction::Cascade,
            "SetNull" => FkAction::SetNull,
            "Restrict" => FkAction::Restrict,
            "SetDefault" => FkAction::SetDefault,
            _ => FkAction::NoAction,
        };
    }
    FkAction::NoAction
}
