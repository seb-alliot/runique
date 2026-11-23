//! Reading the `syn` tree of a SeaORM migration: method chains, string
//! arguments, `Alias::new`, foreign-key references and actions.
use syn::{Expr, ExprCall, ExprLit, ExprMethodCall, Lit};

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

/// Collects every method and function-call name reachable from `expr`, recursing
/// through receivers and arguments — used to detect which builder methods were
/// called anywhere in a column definition, regardless of call order.
pub fn method_names_in_expr(expr: &Expr) -> Vec<String> {
    let mut names = Vec::new();
    collect_method_names(expr, &mut names);
    names
}

fn collect_method_names(expr: &Expr, names: &mut Vec<String>) {
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

/// Searches `expr` (a method-call or function-call chain) for the first string
/// literal argument, checking the receiver before sibling arguments.
pub fn extract_str_from_call(expr: &Expr) -> Option<String> {
    match expr {
        Expr::MethodCall(mc) => {
            if let Some(s) = extract_str_from_call(&mc.receiver) {
                return Some(s);
            }
            for arg in &mc.args {
                if let Some(s) = extract_str_from_call(arg) {
                    return Some(s);
                }
            }
            None
        }
        Expr::Call(ExprCall { args, .. }) => {
            for arg in args {
                if let Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                }) = arg
                {
                    return Some(s.value());
                }
                if let Some(s) = extract_str_from_call(arg) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

/// Collects every string literal reachable from `expr`, including inside
/// `vec![...]` macro invocations (e.g. a list of enum variant names).
pub fn extract_all_str_args(expr: &Expr) -> Vec<String> {
    let mut result = Vec::new();
    collect_str_args(expr, &mut result);
    result
}

fn collect_str_args(expr: &Expr, result: &mut Vec<String>) {
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

/// Finds a `.references(...)` call anywhere in `expr` and returns
/// `(to_table, to_column)`, defaulting the column to `"id"` when only the
/// table is given.
pub fn extract_references_from_expr(expr: &Expr) -> Option<(String, String)> {
    if let Expr::MethodCall(mc) = expr {
        if mc.method == "references" {
            let strings = extract_all_str_args(&mc.args[0]);
            return match strings.len() {
                0 => None,
                1 => Some((strings[0].clone(), "id".to_string())),
                _ => Some((strings[0].clone(), strings[1].clone())),
            };
        }
        if let Some(s) = extract_references_from_expr(&mc.receiver) {
            return Some(s);
        }
        for arg in &mc.args {
            if let Some(s) = extract_references_from_expr(arg) {
                return Some(s);
            }
        }
    }
    None
}

/// Finds a call to `method_name` (e.g. `"on_delete"`/`"on_update"`) anywhere in
/// `expr` and resolves its argument to a `ForeignKeyAction` name. Returns
/// `"NoAction"` if the method isn't called or its argument isn't recognized.
pub fn extract_fk_action(expr: &Expr, method_name: &str) -> String {
    if let Expr::MethodCall(mc) = expr {
        if mc.method == method_name
            && let Some(arg) = mc.args.first()
        {
            return extract_fk_action_value(arg);
        }
        let s = extract_fk_action(&mc.receiver, method_name);
        if s != "NoAction" {
            return s;
        }
        for arg in &mc.args {
            let s = extract_fk_action(arg, method_name);
            if s != "NoAction" {
                return s;
            }
        }
    }
    "NoAction".to_string()
}

/// Maps a path expression (e.g. `ForeignKeyAction::Cascade`) to its action name.
/// Anything unrecognized, including a bare path with no last segment, resolves to `"NoAction"`.
pub fn extract_fk_action_value(expr: &Expr) -> String {
    if let Expr::Path(p) = expr
        && let Some(seg) = p.path.segments.last()
    {
        return match seg.ident.to_string().as_str() {
            "Cascade" => "Cascade".to_string(),
            "SetNull" => "SetNull".to_string(),
            "Restrict" => "Restrict".to_string(),
            _ => "NoAction".to_string(),
        };
    }
    "NoAction".to_string()
}

/// Searches `expr` for a call shaped like `sea_query::Alias::new("name")` (or
/// `Alias::new("name")`) anywhere in a method/function-call chain and returns
/// the literal `"name"`.
pub fn extract_alias_new_str(expr: &Expr) -> Option<String> {
    match expr {
        Expr::MethodCall(mc) => {
            if let Some(s) = extract_alias_new_str(&mc.receiver) {
                return Some(s);
            }
            for arg in &mc.args {
                if let Some(s) = extract_alias_new_str(arg) {
                    return Some(s);
                }
            }
            None
        }
        Expr::Call(ExprCall { func, args, .. }) => {
            let is_alias = if let Expr::Path(p) = func.as_ref() {
                p.path.segments.iter().any(|s| s.ident == "Alias")
            } else {
                false
            };
            if is_alias
                && let Some(Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                })) = args.first()
            {
                return Some(s.value());
            }

            for arg in args {
                if let Some(s) = extract_alias_new_str(arg) {
                    return Some(s);
                }
            }
            None
        }
        _ => None,
    }
}

/// Same as [`extract_alias_new_str`], but only matches when `expr` itself is
/// directly the `Alias::new(...)` call — it does not recurse into a receiver
/// or nested arguments.
pub fn extract_alias_new_str_inner(expr: &Expr) -> Option<String> {
    if let Expr::Call(ExprCall { func, args, .. }) = expr {
        let is_alias = if let Expr::Path(p) = func.as_ref() {
            p.path.segments.iter().any(|s| s.ident == "Alias")
        } else {
            false
        };
        if is_alias
            && let Some(Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            })) = args.first()
        {
            return Some(s.value());
        }
    }
    None
}
