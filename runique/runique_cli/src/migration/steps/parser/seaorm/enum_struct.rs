use super::*;

/// Extracts (enum_name, variants) from `ColumnType::Enum { name: Alias::new("X").into_iden(), variants: vec![...] }`
/// by recursively searching in the expression.
pub(super) fn extract_column_type_enum_struct(expr: &Expr) -> (Option<String>, Vec<String>) {
    match expr {
        Expr::Struct(s) => {
            // ColumnType::Enum { ... }
            if s.path
                .segments
                .last()
                .map(|seg| seg.ident == "Enum")
                .unwrap_or(false)
            {
                let mut enum_name: Option<String> = None;
                let mut variants: Vec<String> = Vec::new();
                for field in &s.fields {
                    let is_name_field = matches!(
                        &field.member,
                        syn::Member::Named(ident) if ident == "name"
                    );
                    let is_variants_field = matches!(
                        &field.member,
                        syn::Member::Named(ident) if ident == "variants"
                    );
                    if is_name_field {
                        enum_name = extract_alias_new_str(&field.expr);
                    } else if is_variants_field {
                        variants = extract_all_str_args(&field.expr);
                    }
                }
                return (enum_name, variants);
            }
            (None, Vec::new())
        }
        Expr::MethodCall(mc) => {
            let res = extract_column_type_enum_struct(&mc.receiver);
            if res.0.is_some() {
                return res;
            }
            for arg in &mc.args {
                let res = extract_column_type_enum_struct(arg);
                if res.0.is_some() {
                    return res;
                }
            }
            (None, Vec::new())
        }
        Expr::Call(syn::ExprCall { func, args, .. }) => {
            let res = extract_column_type_enum_struct(func);
            if res.0.is_some() {
                return res;
            }
            for arg in args {
                let res = extract_column_type_enum_struct(arg);
                if res.0.is_some() {
                    return res;
                }
            }
            (None, Vec::new())
        }
        // `if <Postgres> { ColumnType::Enum { … } } else { ColumnType::String(…) }`
        Expr::If(e) => match e.then_branch.stmts.last() {
            Some(syn::Stmt::Expr(inner, None)) => extract_column_type_enum_struct(inner),
            _ => (None, Vec::new()),
        },
        _ => (None, Vec::new()),
    }
}
