//! `Model Meta` for a model's entity: what the admin needs to know about it.

use crate::model::ast::ModelInput;
use crate::model::utils::ident_pascal;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_model_meta(model: &ModelInput) -> TokenStream2 {
    let key = &model.table;
    let owner = model
        .meta
        .as_ref()
        .and_then(|m| m.owner.as_ref())
        .map(|field| {
            let column = ident_pascal(field);
            quote! {
                const OWNER: Option<Column> = Some(Column::#column);
            }
        });
    quote! {
        impl :: runique::admin::refonte_admin::meta::ModelMeta for Entity {
            const KEY: &'static str = #key;
            #owner
        }
    }
}
