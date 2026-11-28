//! AST Parser for generated SeaORM snapshots — extracts the `ParsedSchema` from snapshot files.
mod column_decimal;
mod column_default;
mod column_length;
mod enum_struct;
mod enum_type_info;
mod index;
mod is_column_def;
mod key;
mod table;

use anyhow::Result;
use quote::ToTokens;
use syn::{Expr, visit::Visit};

use crate::migration::{
    steps::parser::syn_extract::{
        collect_chain, extract_alias_new_str, extract_alias_new_str_inner, extract_all_str_args,
        extract_fk_action, extract_fk_action_value, extract_references_from_expr,
        extract_str_from_call, get_root_expr, method_names_in_expr,
    },
    types::{ColumnType, FkAction, ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema},
};

/// Parses SeaORM source code and returns an analyzed schema.
///
/// # Example
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
        if method == "table"
            && self.table_name.is_none()
            && let Some(arg) = mc.args.first()
        {
            let name = extract_alias_new_str_inner(arg).or_else(|| extract_str_from_call(arg));

            if let Some(n) = name {
                self.table_name = Some(n);
            }
        }
        if method == "col"
            && let Some(arg) = mc.args.first()
            && is_column_def(arg)
        {
            let methods = method_names_in_expr(arg);
            let name = extract_alias_new_str(arg).or_else(|| extract_str_from_call(arg));
            if let Some(n) = name {
                let is_pk = methods.contains(&"primary_key".to_string());
                let col_type = ColumnType::from_methods(&methods);
                let nullable = methods.contains(&"null".to_string());
                let unique = methods.contains(&"unique".to_string())
                    || methods.contains(&"unique_key".to_string());
                // Only a timestamp default sets `has_default_now`; a literal default
                // (`.default(0)`, `.default(true)`, …) is kept separately so it round-trips
                // through the snapshot without being mistaken for a CURRENT_TIMESTAMP default.
                let default_arg = extract_default_arg(arg);
                let is_ts_default = default_arg
                    .as_deref()
                    .is_some_and(|s| s.contains("current_timestamp"));
                let default_value = if is_ts_default { None } else { default_arg };
                let (enum_name, enum_string_values) = if methods.contains(&"enum_type".to_string())
                {
                    extract_enum_type_info(arg)
                } else {
                    // ColumnDef::new_with_type(..., ColumnType::Enum { name: ..., variants: [...] })
                    extract_column_type_enum_struct(arg)
                };
                if is_pk {
                    self.primary_key = Some(ParsedColumn {
                        name: n,
                        col_type,
                        nullable: false,
                        unique: false,
                        ignored: false,
                        has_default_now: false,
                        default_value: None,
                        enum_name: None,
                        enum_string_values: Vec::new(),
                        renamed_from: None,
                        max_length: None,
                        decimal_len: None,
                    });
                } else {
                    self.columns.push(ParsedColumn {
                        max_length: extract_length_arg(arg),
                        decimal_len: extract_decimal_len_arg(arg),
                        name: n,
                        col_type,
                        nullable,
                        unique,
                        ignored: false,
                        has_default_now: is_ts_default,
                        default_value,
                        enum_name,
                        enum_string_values,
                        renamed_from: None,
                    });
                }
            }
        }

        if method == "create_foreign_key"
            && let Some(arg) = mc.args.first()
            && let Some(fk) = extract_seaorm_fk(arg)
        {
            self.foreign_keys.push(fk);
        }

        if method == "create_index"
            && let Some(arg) = mc.args.first()
            && let Some(idx) = extract_seaorm_index(arg)
        {
            self.indexes.push(idx);
        }

        if method == "foreign_key"
            && let Some(arg) = mc.args.first()
        {
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

        if method == "index"
            && let Some(arg) = mc.args.first()
        {
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

use column_decimal::extract_decimal_len_arg;
use column_default::extract_default_arg;
use column_length::extract_length_arg;
use enum_struct::extract_column_type_enum_struct;
use enum_type_info::extract_enum_type_info;
use index::extract_seaorm_index;
use is_column_def::is_column_def;
use key::extract_seaorm_fk;
pub use table::parse_seaorm_source;
