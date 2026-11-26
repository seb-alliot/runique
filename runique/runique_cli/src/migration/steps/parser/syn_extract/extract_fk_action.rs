use super::*;

/// Finds a call to `method_name` (e.g. `"on_delete"`/`"on_update"`) anywhere in
/// `expr` and resolves its argument to a foreign key action. Returns
/// `NoAction` if the method isn't called or its argument isn't recognized.
pub fn extract_fk_action(expr: &Expr, method_name: &str) -> FkAction {
    if let Expr::MethodCall(mc) = expr {
        if mc.method == method_name
            && let Some(arg) = mc.args.first()
        {
            return extract_fk_action_value(arg);
        }
        let action = extract_fk_action(&mc.receiver, method_name);
        if action != FkAction::NoAction {
            return action;
        }
        for arg in &mc.args {
            let action = extract_fk_action(arg, method_name);
            if action != FkAction::NoAction {
                return action;
            }
        }
    }
    FkAction::NoAction
}
