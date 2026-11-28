//! `Model Meta` for a model's entity: what the admin needs to know about it.

use crate::model::{
    ast::{FormFieldType, ModelInput},
    utils::ident_pascal,
};
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
    let label = model
        .meta
        .as_ref()
        .and_then(|m| m.label.as_ref())
        .map(|field| {
            let column = ident_pascal(field);
            quote! {
                const LABEL: Option<Column>= Some(Column::#column);
            }
        });
    // The DSL knows which fields hold a password: the admin never touches them.
    let secrets: Vec<_> = model
        .fields
        .iter()
        .filter(|f| f.field_type == FormFieldType::Password)
        .map(|f| ident_pascal(&f.name))
        .collect();
    let secret = (!secrets.is_empty()).then(|| {
        quote! {
            const SECRET: &'static [Column] = &[#(Column::#secrets),*];
        }
    });
    // The list fields: the gate writes them through the functions the
    // model already has (`admin_save_lists`, `admin_list_values`).
    let list_names: Vec<String> = model.lists.iter().map(|l| l.name.to_string()).collect();
    let list_idents: Vec<_> = model.lists.iter().map(|l| l.name.clone()).collect();
    let lists = (!list_names.is_empty()).then(|| {
        quote! {
            const LISTS: &'static [&'static str] = &[#(#list_names),*];

            fn save_lists<C>(
                row: &Model,
                db: &C,
                values: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            ) -> impl ::std::future::Future<Output = ::std::result::Result<(), ::sea_orm::DbErr>> + ::std::marker::Send
            where
                C: ::sea_orm::ConnectionTrait + ::sea_orm::TransactionTrait,
            {
                row.admin_save_lists(db, values)
            }

            fn list_values<C>(
                key: <<Entity as ::sea_orm::EntityTrait>::PrimaryKey as ::sea_orm::PrimaryKeyTrait>::ValueType,
                db: &C,
            ) -> impl ::std::future::Future<
                Output = ::std::result::Result<
                    ::std::vec::Vec<(&'static str, ::std::string::String)>,
                    ::sea_orm::DbErr,
                >,
            > + ::std::marker::Send
            where
                C: ::sea_orm::ConnectionTrait,
            {
                async move {
                    use ::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
                    let mut read = ::std::vec::Vec::new();
                    #(
                        let values = #list_idents::Entity::find()
                            .filter(#list_idents::Column::OwnerId.eq(key.clone()))
                            .order_by_asc(#list_idents::Column::Id)
                            .all(db)
                            .await?;
                        read.push((
                            #list_names,
                            values
                                .iter()
                                .map(|row| row.value.form_value())
                                .collect::<::std::vec::Vec<_>>()
                                .join(","),
                        ));
                    )*
                    ::std::result::Result::Ok(read)
                }
            }
        }
    });
    quote! {
        impl ::runique::auth::meta::ModelMeta for Entity {
            const KEY: &'static str = #key;
            type Form = Form;
            fn schema() -> ::runique::schema::ModelSchema {
                self::schema()
            }
            #owner
            #label
            #secret
            #lists
        }
    }
}
