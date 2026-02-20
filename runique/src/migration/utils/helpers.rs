use syn::{Expr, ExprCall, ExprLit, ExprMethodCall, Lit};

// ============================================================
// Type mapping
// ============================================================

/// Retourne le nom de la méthode associée à un type de colonne.
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::helpers::col_type_to_method;
/// assert_eq!(col_type_to_method("Text"), "text()");
/// assert_eq!(col_type_to_method("TinyInteger"), "tiny_integer()");
/// assert_eq!(col_type_to_method("Inconnu"), "string()");
/// ```
pub fn col_type_to_method(col_type: &str) -> &str {
    match col_type {
        "Text" => "text()",
        "TinyInteger" => "tiny_integer()",
        "SmallInteger" => "small_integer()",
        "Integer" => "integer()",
        "BigInteger" => "big_integer()",
        "Unsigned" => "unsigned()",
        "BigUnsigned" => "big_unsigned()",
        "Float" => "float()",
        "Double" => "double()",
        "Decimal" => "decimal()",
        "Boolean" => "boolean()",
        "DateTime" => "date_time()",
        "Timestamp" => "timestamp()",
        "TimestampWithTimeZone" => "timestamp_tz()",
        "Date" => "date()",
        "Time" => "time()",
        "Uuid" => "uuid()",
        "Json" => "json()",
        "JsonBinary" => "json_binary()",
        "Binary" => "binary()",
        "VarBinary" => "var_binary()",
        "Blob" => "blob()",
        "Char" => "char()",
        "Inet" => "inet()",
        "Cidr" => "cidr()",
        "MacAddr" => "mac_address()",
        "Interval" => "interval()",
        "Enum" => "enum_type()",
        _ => "string()",
    }
}

pub fn detect_col_type_builder(methods: &[String]) -> String {
    // Binaires
    if methods.contains(&"blob".to_string()) {
        "Blob".to_string()
    } else if methods.contains(&"binary".to_string()) || methods.contains(&"binary_len".to_string())
    {
        "Binary".to_string()
    } else if methods.contains(&"var_binary".to_string()) {
        "VarBinary".to_string()
    }
    // Textes
    else if methods.contains(&"text".to_string()) {
        "Text".to_string()
    } else if methods.contains(&"char".to_string()) || methods.contains(&"char_len".to_string()) {
        "Char".to_string()
    } else if methods.contains(&"varchar".to_string())
        || methods.contains(&"string_len".to_string())
    {
        "String".to_string()
    }
    // Entiers
    else if methods.contains(&"tiny_integer".to_string()) {
        "TinyInteger".to_string()
    } else if methods.contains(&"small_integer".to_string()) {
        "SmallInteger".to_string()
    } else if methods.contains(&"big_unsigned".to_string()) {
        "BigUnsigned".to_string()
    } else if methods.contains(&"unsigned".to_string()) {
        "Unsigned".to_string()
    } else if methods.contains(&"big_integer".to_string()) {
        "BigInteger".to_string()
    } else if methods.contains(&"integer".to_string()) {
        "Integer".to_string()
    }
    // Numériques
    else if methods.contains(&"float".to_string()) {
        "Float".to_string()
    } else if methods.contains(&"double".to_string()) {
        "Double".to_string()
    } else if methods.contains(&"decimal".to_string())
        || methods.contains(&"decimal_len".to_string())
    {
        "Decimal".to_string()
    }
    // Booléen
    else if methods.contains(&"boolean".to_string()) {
        "Boolean".to_string()
    }
    // Date/Heure
    else if methods.contains(&"timestamp_tz".to_string()) {
        "TimestampWithTimeZone".to_string()
    } else if methods.contains(&"timestamp".to_string()) {
        "Timestamp".to_string()
    } else if methods.contains(&"datetime".to_string())
        || methods.contains(&"auto_now".to_string())
        || methods.contains(&"auto_now_update".to_string())
    {
        "DateTime".to_string()
    } else if methods.contains(&"date".to_string()) {
        "Date".to_string()
    } else if methods.contains(&"time".to_string()) {
        "Time".to_string()
    }
    // UUID
    else if methods.contains(&"uuid".to_string()) {
        "Uuid".to_string()
    }
    // JSON
    else if methods.contains(&"json_binary".to_string()) {
        "JsonBinary".to_string()
    } else if methods.contains(&"json".to_string()) {
        "Json".to_string()
    }
    // Fallback
    else {
        "String".to_string()
    }
}

pub fn detect_col_type_seaorm(methods: &[String]) -> String {
    // Binaires
    if methods.contains(&"blob".to_string()) {
        "Blob".to_string()
    } else if methods.contains(&"binary".to_string()) || methods.contains(&"binary_len".to_string())
    {
        "Binary".to_string()
    } else if methods.contains(&"var_binary".to_string()) {
        "VarBinary".to_string()
    }
    // Textes
    else if methods.contains(&"text".to_string()) {
        "Text".to_string()
    } else if methods.contains(&"char".to_string()) || methods.contains(&"char_len".to_string()) {
        "Char".to_string()
    }
    // Entiers (ordre : du plus spécifique au plus général)
    else if methods.contains(&"tiny_integer".to_string()) {
        "TinyInteger".to_string()
    } else if methods.contains(&"small_integer".to_string()) {
        "SmallInteger".to_string()
    } else if methods.contains(&"big_unsigned".to_string()) {
        "BigUnsigned".to_string()
    } else if methods.contains(&"unsigned".to_string()) {
        "Unsigned".to_string()
    } else if methods.contains(&"big_integer".to_string()) {
        "BigInteger".to_string()
    } else if methods.contains(&"integer".to_string()) {
        "Integer".to_string()
    }
    // Numériques
    else if methods.contains(&"float".to_string()) {
        "Float".to_string()
    } else if methods.contains(&"double".to_string()) {
        "Double".to_string()
    } else if methods.contains(&"decimal".to_string())
        || methods.contains(&"decimal_len".to_string())
    {
        "Decimal".to_string()
    }
    // Booléen
    else if methods.contains(&"boolean".to_string()) {
        "Boolean".to_string()
    }
    // Date/Heure
    else if methods.contains(&"timestamp_tz".to_string())
        || methods.contains(&"timestamp_with_time_zone".to_string())
    {
        "TimestampWithTimeZone".to_string()
    } else if methods.contains(&"timestamp".to_string()) {
        "Timestamp".to_string()
    } else if methods.contains(&"date_time".to_string())
        || methods.contains(&"auto_now".to_string())
        || methods.contains(&"auto_now_update".to_string())
    {
        "DateTime".to_string()
    } else if methods.contains(&"date".to_string()) {
        "Date".to_string()
    } else if methods.contains(&"time".to_string()) {
        "Time".to_string()
    }
    // UUID
    else if methods.contains(&"uuid".to_string()) {
        "Uuid".to_string()
    }
    // JSON
    else if methods.contains(&"json_binary".to_string()) {
        "JsonBinary".to_string()
    } else if methods.contains(&"json".to_string()) {
        "Json".to_string()
    }
    // PostgreSQL spécifiques
    else if methods.contains(&"inet".to_string()) {
        "Inet".to_string()
    } else if methods.contains(&"cidr".to_string()) {
        "Cidr".to_string()
    } else if methods.contains(&"mac_address".to_string()) {
        "MacAddr".to_string()
    } else if methods.contains(&"interval".to_string()) {
        "Interval".to_string()
    }
    // Enum
    else if methods.contains(&"enum_type".to_string())
        || methods.contains(&"enumeration".to_string())
    {
        "Enum".to_string()
    }
    // Fallback
    else {
        "String".to_string()
    }
}

// ============================================================
// String helpers
// ============================================================

/// Convertit une chaîne PascalCase en snake_case.
///
/// # Exemple
///
/// ```rust
/// use runique::migration::utils::helpers::to_snake_case;
/// assert_eq!(to_snake_case("PascalCase"), "pascal_case");
/// assert_eq!(to_snake_case("Test"), "test");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

// ============================================================
// AST extraction helpers
// ============================================================

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

pub fn first_str_arg(mc: &ExprMethodCall) -> Option<String> {
    if let Some(Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    })) = mc.args.first()
    {
        Some(s.value())
    } else {
        None
    }
}

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
        _ => {}
    }
}

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

pub fn extract_fk_action(expr: &Expr, method_name: &str) -> String {
    if let Expr::MethodCall(mc) = expr {
        if mc.method == method_name {
            if let Some(arg) = mc.args.first() {
                return extract_fk_action_value(arg);
            }
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

pub fn extract_fk_action_value(expr: &Expr) -> String {
    if let Expr::Path(p) = expr {
        if let Some(seg) = p.path.segments.last() {
            return match seg.ident.to_string().as_str() {
                "Cascade" => "Cascade".to_string(),
                "SetNull" => "SetNull".to_string(),
                "Restrict" => "Restrict".to_string(),
                _ => "NoAction".to_string(),
            };
        }
    }
    "NoAction".to_string()
}

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
            if is_alias {
                if let Some(Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                })) = args.first()
                {
                    return Some(s.value());
                }
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

pub fn extract_alias_new_str_inner(expr: &Expr) -> Option<String> {
    if let Expr::Call(ExprCall { func, args, .. }) = expr {
        let is_alias = if let Expr::Path(p) = func.as_ref() {
            p.path.segments.iter().any(|s| s.ident == "Alias")
        } else {
            false
        };
        if is_alias {
            if let Some(Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            })) = args.first()
            {
                return Some(s.value());
            }
        }
    }
    None
}
