use anyhow::Result;
use syn::{visit::Visit, Expr};

use crate::migration::utils::{
    helpers::{
        collect_chain, detect_col_type_seaorm, extract_alias_new_str, extract_alias_new_str_inner,
        extract_all_str_args, extract_fk_action, extract_fk_action_value,
        extract_references_from_expr, extract_str_from_call, method_names_in_expr,
    },
    types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

/// Parse le code source SeaORM et retourne un schéma analysé.
///
/// # Exemple
///
/// ```rust,ignore
/// // let schema = parse_seaorm_source("...");
/// // assert!(schema.is_ok());
/// ```

struct SeaOrmVisitor {
    pub table_name: Option<String>,
    pub primary_key: Option<ParsedColumn>,
    pub columns: Vec<ParsedColumn>,
    pub foreign_keys: Vec<ParsedFk>,
    pub indexes: Vec<ParsedIndex>,
    in_up: bool,
}

impl SeaOrmVisitor {
    fn new() -> Self {
        Self {
            table_name: None,
            primary_key: None,
            columns: Vec::new(),
            foreign_keys: Vec::new(),
            indexes: Vec::new(),
            in_up: false,
        }
    }
}

impl<'ast> Visit<'ast> for SeaOrmVisitor {
    fn visit_impl_item_fn(&mut self, f: &'ast syn::ImplItemFn) {
        if f.sig.ident == "up" {
            self.in_up = true;
            syn::visit::visit_impl_item_fn(self, f);
            self.in_up = false;
        }
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.in_up {
            self.try_extract(expr);
        }
        syn::visit::visit_expr(self, expr);
    }
}

impl SeaOrmVisitor {
    fn try_extract(&mut self, expr: &Expr) {
        let mc = if let Expr::MethodCall(mc) = expr {
            mc
        } else {
            return;
        };
        let method = mc.method.to_string();

        if method == "col" {
            if let Some(arg) = mc.args.first() {
                let methods = method_names_in_expr(arg);
                let name = extract_alias_new_str(arg).or_else(|| extract_str_from_call(arg));
                if let Some(n) = name {
                    let is_pk = methods.contains(&"primary_key".to_string());
                    let col_type = detect_col_type_seaorm(&methods);
                    let nullable = methods.contains(&"null".to_string());
                    let unique = methods.contains(&"unique".to_string());
                    if is_pk {
                        self.primary_key = Some(ParsedColumn {
                            name: n,
                            col_type,
                            nullable: false,
                            unique: false,
                            ignored: false,
                        });
                    } else {
                        self.columns.push(ParsedColumn {
                            name: n,
                            col_type,
                            nullable,
                            unique,
                            ignored: false,
                        });
                    }
                }
            }
        }

        if method == "create_foreign_key" {
            if let Some(arg) = mc.args.first() {
                if let Some(fk) = extract_seaorm_fk(arg) {
                    self.foreign_keys.push(fk);
                }
            }
        }

        if method == "create_index" {
            if let Some(arg) = mc.args.first() {
                if let Some(idx) = extract_seaorm_index(arg) {
                    self.indexes.push(idx);
                }
            }
        }

        if method == "foreign_key" {
            if let Some(arg) = mc.args.first() {
                let from_column = extract_alias_new_str(arg)
                    .or_else(|| extract_str_from_call(arg))
                    .unwrap_or_default();
                let (to_table, to_column) = extract_references_from_expr(arg)
                    .unwrap_or_else(|| ("".to_string(), "id".to_string()));
                let on_delete = extract_fk_action(arg, "on_delete");
                let on_update = extract_fk_action(arg, "on_update");
                self.foreign_keys.push(ParsedFk {
                    from_column,
                    to_table,
                    to_column,
                    on_delete,
                    on_update,
                });
            }
        }

        if method == "index" {
            if let Some(arg) = mc.args.first() {
                let methods = method_names_in_expr(arg);
                let strings = extract_all_str_args(arg);
                let unique = methods.contains(&"unique".to_string());
                if let Some(name) = strings.first() {
                    self.indexes.push(ParsedIndex {
                        name: name.clone(),
                        columns: strings[1..].to_vec(),
                        unique,
                    });
                }
            }
        }
    }
}

fn extract_seaorm_fk(expr: &Expr) -> Option<ParsedFk> {
    let chain = collect_chain(expr);
    let mut from_col: Option<String> = None;
    let mut to_table: Option<String> = None;
    let mut to_col: Option<String> = None;
    let mut on_delete = "NoAction".to_string();
    let mut on_update = "NoAction".to_string();

    for mc in &chain {
        match mc.method.to_string().as_str() {
            "from" if mc.args.len() >= 2 => {
                from_col = extract_alias_new_str_inner(&mc.args[1]);
            }
            "to" if mc.args.len() >= 2 => {
                to_table = extract_alias_new_str_inner(&mc.args[0]);
                to_col = extract_alias_new_str_inner(&mc.args[1]);
            }
            "on_delete" => {
                if let Some(arg) = mc.args.first() {
                    on_delete = extract_fk_action_value(arg);
                }
            }
            "on_update" => {
                if let Some(arg) = mc.args.first() {
                    on_update = extract_fk_action_value(arg);
                }
            }
            _ => {}
        }
    }

    Some(ParsedFk {
        from_column: from_col?,
        to_table: to_table?,
        to_column: to_col.unwrap_or_else(|| "id".to_string()),
        on_delete,
        on_update,
    })
}

fn extract_seaorm_index(expr: &Expr) -> Option<ParsedIndex> {
    let chain = collect_chain(expr);
    let mut name: Option<String> = None;
    let mut columns = Vec::new();
    let mut unique = false;

    for mc in &chain {
        match mc.method.to_string().as_str() {
            "name" => {
                if let Some(syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                })) = mc.args.first()
                {
                    name = Some(s.value());
                }
            }
            "col" => {
                if let Some(arg) = mc.args.first() {
                    if let Some(col) =
                        extract_alias_new_str_inner(arg).or_else(|| extract_str_from_call(arg))
                    {
                        columns.push(col);
                    }
                }
            }
            "unique" => {
                unique = true;
            }
            _ => {}
        }
    }

    Some(ParsedIndex {
        name: name?,
        columns,
        unique,
    })
}

pub fn parse_seaorm_source(source: &str) -> Result<ParsedSchema> {
    let file =
        syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    let mut visitor = SeaOrmVisitor::new();
    visitor.visit_file(&file);
    let table_name = visitor
        .table_name
        .ok_or_else(|| anyhow::anyhow!("Cannot extract table name"))?;
    Ok(ParsedSchema {
        table_name,
        primary_key: visitor.primary_key,
        columns: visitor.columns,
        foreign_keys: visitor.foreign_keys,
        indexes: visitor.indexes,
    })
}
