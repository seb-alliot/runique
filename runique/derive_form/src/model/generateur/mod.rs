//! Code generation for `model!{}` — one file per thing generated.
mod admin_form;
mod conversion;
mod enums;
mod lists;
mod schema;

pub use admin_form::*;
pub use conversion::*;
pub use enums::*;
pub use lists::*;
pub use schema::*;

use crate::model::ast::*;
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate(model: &ModelInput) -> TokenStream2 {
    let enums = generate_enums(model);
    let schema = generate_schema(model);
    let sea_model = generate_sea_model(model);
    let relation_enum = generate_relation_enum(model);
    let active_model = generate_active_model(model);
    let from_str_map: TokenStream2 = generate_from_str_map(model);
    let partial_update: TokenStream2 = generate_partial_update(model);
    let admin_form = generate_admin_form(model);
    let unique_fields = generate_unique_fields(model);
    let enum_labels = generate_enum_label_resolver(model);
    let lists = generate_lists(model);
    let admin_lists = generate_admin_lists(&model.lists);

    quote! {
        #enums
        #schema
        #sea_model
        #relation_enum
        #active_model
        #from_str_map
        #partial_update
        #admin_form
        #unique_fields
        #enum_labels
        #lists
        #admin_lists
    }
}
