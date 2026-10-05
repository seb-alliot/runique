use crate::model::generateur::{generate_column, generate_pk};
use crate::model::utils::relation_enum::{fk_action_tokens, target_entity, target_pk_column};
use crate::model::{ModelInput, RelationDef};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── FK dans generate_schema ───────────────────────────────────

pub fn generate_schema(model: &ModelInput) -> TokenStream2 {
    let name = &model.name;
    let table = &model.table;
    let pk = generate_pk(&model.pk);
    let columns: Vec<TokenStream2> = model
        .fields
        .iter()
        .map(|f| generate_column(f, &model.enums))
        .collect();
    let fks: Vec<TokenStream2> = generate_foreign_keys(model);
    let meta = generate_meta(model);

    quote! {
        pub fn schema() -> ::runique::migration::schema::ModelSchema {
            ::runique::migration::ModelSchema::new(stringify!(#name))
                .table_name(#table)
                #pk
                #(#columns)*
                #(#fks)*
                #meta
                .build()
                .unwrap()
        }
    }
}

/// One FK constraint per `belongs_to`: its column references the target
/// entity's table and primary key, read from the entity itself.
fn generate_foreign_keys(model: &ModelInput) -> Vec<TokenStream2> {
    model
        .relations
        .iter()
        .filter_map(|rel| {
            let RelationDef::BelongsTo {
                model: target,
                via,
                on_delete,
                on_update,
            } = rel
            else {
                return None;
            };
            let via = via.to_string();
            let entity = target_entity(target);
            let pk_column = target_pk_column(target);
            let on_delete = fk_action_tokens(*on_delete);
            let on_update = fk_action_tokens(*on_update);
            Some(quote! {
                .foreign_key(
                    ::runique::migration::ForeignKeyDef::new(#via)
                        .references(::sea_orm::EntityName::table_name(&#entity))
                        .to_column(::sea_orm::IdenStatic::as_str(&#pk_column))
                        .on_delete(#on_delete)
                        .on_update(#on_update)
                )
            })
        })
        .collect()
}

fn generate_meta(model: &ModelInput) -> TokenStream2 {
    let Some(meta) = &model.meta else {
        return quote! {};
    };

    let ordering: Vec<TokenStream2> = meta
        .ordering
        .iter()
        .map(|(desc, field)| {
            let field_str = field.to_string();
            if *desc {
                quote! { .order_by(#field_str, ::runique::migration::OrderDir::Desc) }
            } else {
                quote! { .order_by(#field_str, ::runique::migration::OrderDir::Asc) }
            }
        })
        .collect();

    let unique_together: Vec<TokenStream2> = meta
        .unique_together
        .iter()
        .map(|group| {
            let fields: Vec<String> = group.iter().map(|f| f.to_string()).collect();
            quote! { .unique_together(vec![#(#fields.to_string()),*]) }
        })
        .collect();

    let indexes: Vec<TokenStream2> = meta
        .indexes
        .iter()
        .map(|group| {
            let fields: Vec<String> = group.iter().map(|f| f.to_string()).collect();
            quote! { .index(::runique::migration::IndexDef::new(vec![#(#fields.to_string()),*])) }
        })
        .collect();

    let verbose = meta
        .verbose_name
        .as_ref()
        .map(|v| {
            quote! {
                .verbose_name(#v)
            }
        })
        .unwrap_or_default();

    let verbose_plural = meta
        .verbose_name_plural
        .as_ref()
        .map(|v| {
            quote! {
                .verbose_name_plural(#v)
            }
        })
        .unwrap_or_default();

    quote! {
        #(#ordering)*
        #(#unique_together)*
        #(#indexes)*
        #verbose
        #verbose_plural
    }
}
