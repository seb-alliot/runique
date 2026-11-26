use super::*;

pub(super) fn collect_method_names(expr: &Expr, names: &mut Vec<String>) {
    match expr {
        Expr::MethodCall(mc) => {
            names.push(mc.method.to_string());
            collect_method_names(&mc.receiver, names);
            for arg in &mc.args {
                collect_method_names(arg, names);
            }
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            collect_method_names(func, names);
            for arg in args {
                collect_method_names(arg, names);
            }
        }
        _ => {}
    }
}
