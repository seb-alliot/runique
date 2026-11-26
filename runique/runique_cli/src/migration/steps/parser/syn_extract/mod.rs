//! Reading the `syn` tree of a SeaORM migration: method chains, string
//! arguments, `Alias::new`, foreign-key references and actions.
mod collect_chain;
mod collect_method_names;
mod collect_str_args;
mod extract_alias_new_str;
mod extract_alias_new_str_inner;
mod extract_all_str_args;
mod extract_fk_action;
mod extract_fk_action_value;
mod extract_references_from_expr;
mod extract_str_from_call;
mod get_root_expr;
mod method_names_in_expr;

use crate::migration::types::FkAction;
use syn::{Expr, ExprCall, ExprLit, ExprMethodCall, Lit};

pub use collect_chain::collect_chain;
use collect_method_names::collect_method_names;
use collect_str_args::collect_str_args;
pub use extract_alias_new_str::extract_alias_new_str;
pub use extract_alias_new_str_inner::extract_alias_new_str_inner;
pub use extract_all_str_args::extract_all_str_args;
pub use extract_fk_action::extract_fk_action;
pub use extract_fk_action_value::extract_fk_action_value;
pub use extract_references_from_expr::extract_references_from_expr;
pub use extract_str_from_call::extract_str_from_call;
pub use get_root_expr::get_root_expr;
pub use method_names_in_expr::method_names_in_expr;
