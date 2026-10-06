//! List fields (`checkbox` / `multichoice`): each one gets a SeaORM entity for
//! its table (`{table}_{field}`), methods on the owner's `Model` to read,
//! replace and preload it, and a [`ListField`] in `List` so `search!` and
//! `filter()` can test what it contains.
//!
//! [`ListField`]: https://docs.rs/runique/latest/runique/macros/bdd/list/struct.ListField.html
use crate::model::ast::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
                .unwrap_or_default()
        })
        .collect()
}

/// What the admin needs from a model's list fields, generated on every model
/// and every `extend!{}` (empty without lists) so the admin code can call it
/// whatever the resource: save the submitted values, and add the stored ones
/// to the row it edits.
pub fn generate_admin_lists(lists: &[FormFieldDecl]) -> TokenStream2 {
    let mut saves = Vec::new();
    let mut reads = Vec::new();
    for list in lists {
        let Some(value_ty) = list.attrs.iter().find_map(|a| match a {
            FormFieldAttr::EnumRef(id) => Some(id),
            _ => None,
        }) else {
            continue;
        };
        let field = &list.name;
        let name = field.to_string();
        let set_fn = format_ident!("set_{}", field);
        saves.push(quote! {
            if let ::std::option::Option::Some(raw) = data.get(#name) {
                let values = raw
                    .split(',')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .filter_map(|v| <#value_ty as ::std::str::FromStr>::from_str(v).ok());
                self.#set_fn(db, values).await?;
            }
        });
        reads.push(quote! {
            let values = self.#field(db).await?;
            if let ::std::option::Option::Some(object) = row.as_object_mut() {
                let joined = values.iter().map(|v| v.form_value()).collect::<::std::vec::Vec<_>>().join(",");
                object.insert(#name.to_string(), ::runique::serde_json::Value::String(joined));
            }
        });
    }
    quote! {
        impl Model {
            /// Admin: replaces each list field present in the submitted `data`
            /// (comma-separated values, already validated by the form).
            pub async fn admin_save_lists<C>(
                &self,
                db: &C,
                data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            ) -> ::std::result::Result<(), ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait + ::sea_orm::TransactionTrait,
            {
                let _ = (db, data);
                #(#saves)*
                ::std::result::Result::Ok(())
            }

            /// Admin: adds the stored list values to `row` (the model as JSON),
            /// the way the edit form reads them.
            pub async fn admin_list_values<C>(
                &self,
                db: &C,
                row: &mut ::runique::serde_json::Value,
            ) -> ::std::result::Result<(), ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                let _ = (db, &row);
                #(#reads)*
                ::std::result::Result::Ok(())
            }
        }
    }
}

pub fn generate_lists(model: &ModelInput) -> TokenStream2 {
    if model.lists.is_empty() {
        return quote! {};
    }
    let pk = &model.pk.name;
    let pk_column = format_ident!("{}", pascal_case(&pk.to_string()));
    let pk_ty = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };

    let mut modules = Vec::new();
    let mut methods = Vec::new();
    let mut fields = Vec::new();
    for list in &model.lists {
        let Some(value_ty) = list.attrs.iter().find_map(|a| match a {
            FormFieldAttr::EnumRef(id) => Some(id),
            _ => None,
        }) else {
            continue;
        };
        let field = &list.name;
        let table = format!("{}_{}", model.table, field);
        let set_fn = format_ident!("set_{}", field);
        let load_fn = format_ident!("load_{}", field);

        modules.push(quote! {
            /// The table of the list field: one row per value chosen.
            pub mod #field {
                use ::sea_orm::entity::prelude::*;

                #[derive(Clone, Debug, PartialEq, ::sea_orm::DeriveEntityModel)]
                #[sea_orm(table_name = #table)]
                pub struct Model {
                    #[sea_orm(primary_key)]
                    pub id: i64,
                    pub owner_id: #pk_ty,
                    pub value: super::#value_ty,
                }

                #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter)]
                pub enum Relation {
                    Owner,
                }

                impl ::sea_orm::RelationTrait for Relation {
                    fn def(&self) -> ::sea_orm::RelationDef {
                        match self {
                            Self::Owner => <Entity as ::sea_orm::EntityTrait>::belongs_to(super::Entity)
                                .from(Column::OwnerId)
                                .to(super::Column::#pk_column)
                                .on_delete(::sea_orm::sea_query::ForeignKeyAction::Cascade)
                                .into(),
                        }
                    }
                }

                impl ::sea_orm::Related<super::Entity> for Entity {
                    fn to() -> ::sea_orm::RelationDef {
                        Relation::Owner.def()
                    }
                }

                impl ::sea_orm::ActiveModelBehavior for ActiveModel {}
            }
        });

        methods.push(quote! {
            /// The values of this list field, in the order they were set.
            pub async fn #field<C>(&self, db: &C) -> ::std::result::Result<::std::vec::Vec<#value_ty>, ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                use ::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
                ::std::result::Result::Ok(
                    #field::Entity::find()
                        .filter(#field::Column::OwnerId.eq(self.#pk.clone()))
                        .order_by_asc(#field::Column::Id)
                        .all(db)
                        .await?
                        .into_iter()
                        .map(|row| row.value)
                        .collect(),
                )
            }

            /// Replaces the values of this list field, duplicates dropped, in
            /// one transaction.
            pub async fn #set_fn<C>(
                &self,
                db: &C,
                values: impl ::std::iter::IntoIterator<Item = #value_ty>,
            ) -> ::std::result::Result<(), ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait + ::sea_orm::TransactionTrait,
            {
                use ::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, TransactionSession, TransactionTrait};
                let mut unique: ::std::vec::Vec<#value_ty> = ::std::vec::Vec::new();
                for value in values {
                    if !unique.contains(&value) {
                        unique.push(value);
                    }
                }
                let owner = self.#pk.clone();
                let txn = db.begin().await?;
                #field::Entity::delete_many()
                    .filter(#field::Column::OwnerId.eq(owner.clone()))
                    .exec(&txn)
                    .await?;
                if !unique.is_empty() {
                    #field::Entity::insert_many(unique.into_iter().map(|value| #field::ActiveModel {
                        id: ::sea_orm::ActiveValue::NotSet,
                        owner_id: ::sea_orm::ActiveValue::Set(owner.clone()),
                        value: ::sea_orm::ActiveValue::Set(value),
                    }))
                    .exec(&txn)
                    .await?;
                }
                txn.commit().await
            }

            /// The values of this list field for every model of a page, in
            /// one query — instead of one query per model.
            pub async fn #load_fn<C>(
                db: &C,
                models: &[Model],
            ) -> ::std::result::Result<::std::collections::HashMap<#pk_ty, ::std::vec::Vec<#value_ty>>, ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                use ::sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
                let mut by_owner: ::std::collections::HashMap<#pk_ty, ::std::vec::Vec<#value_ty>> = models
                    .iter()
                    .map(|m| (m.#pk.clone(), ::std::vec::Vec::new()))
                    .collect();
                if by_owner.is_empty() {
                    return ::std::result::Result::Ok(by_owner);
                }
                let rows = #field::Entity::find()
                    .filter(#field::Column::OwnerId.is_in(by_owner.keys().cloned().collect::<::std::vec::Vec<_>>()))
                    .order_by_asc(#field::Column::Id)
                    .all(db)
                    .await?;
                for row in rows {
                    by_owner.entry(row.owner_id).or_default().push(row.value);
                }
                ::std::result::Result::Ok(by_owner)
            }
        });

        // PascalCase, like the `Column` variants: `search!(… => Title eq x, Genres has y)`.
        let const_name = format_ident!("{}", pascal_case(&field.to_string()));
        fields.push(quote! {
            pub const #const_name: ::runique::prelude::ListField<Entity, #field::Column, #value_ty> =
                ::runique::prelude::ListField::new(#field::Column::OwnerId, #field::Column::Value);
        });
    }

    quote! {
        #(#modules)*

        impl Model {
            #(#methods)*
        }

        /// The list fields of this model, for `search!(… => Field has value)`
        /// and `filter(List::Field.has(value))`.
        pub struct List;

        #[allow(non_upper_case_globals)]
        impl List {
            #(#fields)*
        }

        impl ::runique::prelude::HasLists for Entity {
            type List = List;
        }
    }
}
