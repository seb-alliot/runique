use crate::model::generateur::{generate_column, generate_pk};
use crate::model::{FieldOption, FkAction, ModelInput, RelationDef};
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
    let relations: Vec<TokenStream2> = generate_relations(model);
    let meta = generate_meta(model);

    quote! {
        pub fn schema() -> ::runique::migration::schema::ModelSchema {
            ::runique::migration::ModelSchema::new(stringify!(#name))
                .table_name(#table)
                #pk
                #(#columns)*
                #(#fks)*
                #(#relations)*
                #meta
                .build()
                .unwrap()
        }
    }
}

fn generate_foreign_keys(model: &ModelInput) -> Vec<TokenStream2> {
    model
        .fields
        .iter()
        .filter_map(|field| {
            let col_name = field.name.to_string();
            field.options.iter().find_map(|opt| {
                if let FieldOption::Fk(fk) = opt {
                    let table = fk.table.to_string();
                    let column = fk.column.to_string();
                    let action = match fk.action {
                        FkAction::Cascade => quote! { ::sea_query::ForeignKeyAction::Cascade },
                        FkAction::SetNull => quote! { ::sea_query::ForeignKeyAction::SetNull },
                        FkAction::Restrict => quote! { ::sea_query::ForeignKeyAction::Restrict },
                        FkAction::SetDefault => {
                            quote! { ::sea_query::ForeignKeyAction::SetDefault }
                        }
                    };
                    Some(quote! {
                        .foreign_key(
                            ::runique::migration::ForeignKeyDef::new(#col_name)
                                .references(#table)
                                .references_column(#column)
                                .on_delete(#action)
                        )
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

fn generate_relations(model: &ModelInput) -> Vec<TokenStream2> {
    model.relations.iter().map(|rel| {
        match rel {
            RelationDef::BelongsTo { model, via } => {
                let model_str = model.to_string().to_lowercase();
                let via_str = via.to_string();
                quote! {
                    .relation(::runique::migration::RelationDef::belongs_to(#model_str, #via_str, "id"))
                }
            }
            RelationDef::HasMany { model, as_name } => {
                let model_str = model.to_string().to_lowercase();
                let as_str = as_name.as_ref().map(|a| a.to_string()).unwrap_or_default();
                quote! {
                    .relation(::runique::migration::RelationDef::has_many(#model_str).as_name(#as_str))
                }
            }
            RelationDef::HasOne { model, as_name } => {
                let model_str = model.to_string().to_lowercase();
                let as_str = as_name.as_ref().map(|a| a.to_string()).unwrap_or_default();
                quote! {
                    .relation(::runique::migration::RelationDef::has_one(#model_str).as_name(#as_str))
                }
            }
            RelationDef::ManyToMany { model, through, via_self: _ } => {
                let model_str = model.to_string().to_lowercase();
                let through_str = through.to_string();
                quote! {
                    .relation(::runique::migration::RelationDef::many_to_many(#model_str).through(#through_str))
                }
            }
        }
    }).collect()
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
