//! `Model Meta` for a model's entity: what the admin needs to know about it.

use crate::model::ast::{FormFieldKind, ModelInput};
use crate::model::utils::ident_pascal;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_model_meta(model: &ModelInput) -> TokenStream2 {
    // Without `runique/admin`, the admin module (and `ModelMeta`) doesn't exist.
    if !cfg!(feature = "admin") {
        return TokenStream2::new();
    }
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
    // The DSL knows which fields hold a password: the admin never touches them.
    let secrets: Vec<_> = model
        .fields
        .iter()
        .filter(|f| f.kind == FormFieldKind::Password)
        .map(|f| ident_pascal(&f.name))
        .collect();
    let secret = (!secrets.is_empty()).then(|| {
        quote! {
            const SECRET: &'static [Column] = &[#(Column::#secrets),*];
        }
    });
    quote! {
        impl :: runique::admin::refonte_admin::meta::ModelMeta for Entity {
            const KEY: &'static str = #key;
            type AdminForm = AdminForm;
            fn schema() -> ::runique::migration::schema::ModelSchema {
                self::schema()
            }
            #owner
            #secret
        }
    }
}
