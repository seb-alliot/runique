use super::*;

pub(super) fn collect_str_args(expr: &Expr, result: &mut Vec<String>) {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => {
            result.push(s.value());
        }
        Expr::MethodCall(mc) => {
            collect_str_args(&mc.receiver, result);
            for arg in &mc.args {
                collect_str_args(arg, result);
            }
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            collect_str_args(func, result);
            for arg in args {
                collect_str_args(arg, result);
            }
        }
        Expr::Macro(syn::ExprMacro { mac, .. }) => {
            // Handles vec!["a".to_string(), "b".to_string()] and similar
            struct ExprList(syn::punctuated::Punctuated<Expr, syn::Token![,]>);
            impl syn::parse::Parse for ExprList {
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    Ok(ExprList(syn::punctuated::Punctuated::parse_terminated(
                        input,
                    )?))
                }
            }
            if let Ok(ExprList(parsed)) = syn::parse2::<ExprList>(mac.tokens.clone()) {
                for expr in parsed {
                    collect_str_args(&expr, result);
                }
            }
        }
        _ => {}
    }
}
