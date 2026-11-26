use super::*;

/// Flattens a chained method-call expression (`a.b().c().d()`) into its individual
/// calls, in source (left-to-right, i.e. call-order) sequence.
pub fn collect_chain(expr: &Expr) -> Vec<&ExprMethodCall> {
    let mut chain = Vec::new();
    let mut current = expr;
    while let Expr::MethodCall(mc) = current {
        chain.push(mc);
        current = &mc.receiver;
    }
    chain.reverse();
    chain
}
