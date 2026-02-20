use syn::{visit::Visit, Expr, ExprLit, Lit};

use crate::migration::utils::{
    helpers::{
        collect_chain, detect_col_type_builder, extract_all_str_args, extract_fk_action,
        extract_references_from_expr, extract_str_from_call, first_str_arg, get_root_expr,
        method_names_in_expr, to_snake_case,
    },
    types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

struct BuilderVisitor {
    pub schema: Option<ParsedSchema>,
}

impl BuilderVisitor {
    fn new() -> Self {
        Self { schema: None }
    }
}

impl<'ast> Visit<'ast> for BuilderVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.schema.is_none() {
            if let Some(schema) = try_parse_builder_chain(expr) {
                self.schema = Some(schema);
            }
        }
        syn::visit::visit_expr(self, expr);
    }
}

fn try_parse_builder_chain(expr: &Expr) -> Option<ParsedSchema> {
    let chain = collect_chain(expr);
    if chain.is_empty() {
        return None;
    }

    let has_build = chain.iter().any(|mc| mc.method == "build");
    if !has_build {
        return None;
    }

    let mut table_name: Option<String> = None;
    let mut primary_key: Option<ParsedColumn> = None;
    let mut columns: Vec<ParsedColumn> = Vec::new();
    let mut foreign_keys: Vec<ParsedFk> = Vec::new();
    let mut indexes: Vec<ParsedIndex> = Vec::new();

    let root = get_root_expr(expr);
    if let Expr::Macro(m) = root {
        if let Some(seg) = m.mac.path.segments.last() {
            if seg.ident == "model" {
                if let Ok(Expr::Lit(ExprLit {
                    lit: Lit::Str(s), ..
                })) = m.mac.parse_body::<Expr>()
                {
                    table_name = Some(to_snake_case(&s.value()));
                }
            }
        }
    }

    for mc in &chain {
        let method = mc.method.to_string();
        match method.as_str() {
            "table_name" => {
                if let Some(name) = first_str_arg(mc) {
                    table_name = Some(name);
                }
            }
            "primary_key" => {
                if let Some(arg) = mc.args.first() {
                    let methods = method_names_in_expr(arg);
                    let name = extract_str_from_call(arg);
                    let col_type = if methods.contains(&"uuid".to_string()) {
                        "Uuid".to_string()
                    } else if methods.contains(&"i64".to_string())
                        || methods.contains(&"big_integer".to_string())
                    {
                        "BigInteger".to_string()
                    } else if methods.contains(&"i32".to_string())
                        || methods.contains(&"integer".to_string())
                    {
                        "Integer".to_string()
                    } else if methods.contains(&"i16".to_string())
                        || methods.contains(&"small_integer".to_string())
                    {
                        "SmallInteger".to_string()
                    } else if methods.contains(&"i8".to_string())
                        || methods.contains(&"tiny_integer".to_string())
                    {
                        "TinyInteger".to_string()
                    } else if methods.contains(&"u64".to_string())
                        || methods.contains(&"big_unsigned".to_string())
                    {
                        "BigUnsigned".to_string()
                    } else if methods.contains(&"u32".to_string())
                        || methods.contains(&"unsigned".to_string())
                    {
                        "Unsigned".to_string()
                    } else if methods.contains(&"string".to_string())
                        || methods.contains(&"varchar".to_string())
                    {
                        "String".to_string()
                    } else {
                        "Integer".to_string()
                    };
                    if let Some(n) = name {
                        primary_key = Some(ParsedColumn {
                            name: n,
                            col_type: col_type.to_string(),
                            nullable: false,
                            unique: false,
                            ignored: false,
                        });
                    }
                }
            }
            "column" => {
                if let Some(arg) = mc.args.first() {
                    let methods = method_names_in_expr(arg);
                    let name = extract_str_from_call(arg);
                    if let Some(n) = name {
                        let col_type = detect_col_type_builder(&methods);
                        let nullable = methods.contains(&"nullable".to_string())
                            || methods.contains(&"auto_now".to_string())
                            || methods.contains(&"auto_now_update".to_string());
                        let unique = methods.contains(&"unique".to_string());
                        let ignored = methods.contains(&"ignored".to_string());
                        columns.push(ParsedColumn {
                            name: n,
                            col_type,
                            nullable,
                            unique,
                            ignored,
                        });
                    }
                }
            }
            "foreign_key" => {
                if let Some(arg) = mc.args.first() {
                    let from_column = extract_str_from_call(arg).unwrap_or_default();
                    let (to_table, to_column) = extract_references_from_expr(arg)
                        .unwrap_or_else(|| ("".to_string(), "id".to_string()));
                    let on_delete = extract_fk_action(arg, "on_delete");
                    let on_update = extract_fk_action(arg, "on_update");
                    foreign_keys.push(ParsedFk {
                        from_column,
                        to_table,
                        to_column,
                        on_delete,
                        on_update,
                    });
                }
            }
            "index" => {
                if let Some(arg) = mc.args.first() {
                    let methods = method_names_in_expr(arg);
                    let strings = extract_all_str_args(arg);
                    let unique = methods.contains(&"unique".to_string());
                    if let Some(name) = strings.first() {
                        indexes.push(ParsedIndex {
                            name: name.clone(),
                            columns: strings[1..].to_vec(),
                            unique,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    let table_name = table_name?;
    Some(ParsedSchema {
        table_name,
        primary_key,
        columns,
        foreign_keys,
        indexes,
    })
}

pub fn parse_schema_from_source(source: &str) -> Option<ParsedSchema> {
    let file = syn::parse_str::<syn::File>(source).ok()?;
    let mut visitor = BuilderVisitor::new();
    visitor.visit_file(&file);
    visitor.schema
}
