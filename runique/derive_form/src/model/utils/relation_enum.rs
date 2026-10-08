use crate::model::{FkAction, ModelInput, RelationDef};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Tables provided by the framework — their SeaORM entities are in `runique`, not in `super::`.
const FRAMEWORK_TABLES: &[(&str, &str)] = &[
    ("eihwaz_users", "::runique::auth::user"),
    ("eihwaz_groupes", "::runique::auth::permissions::groupe"),
    (
        "eihwaz_groupes_droits",
        "::runique::auth::permissions::groupes_droits",
    ),
    (
        "eihwaz_users_groupes",
        "::runique::auth::permissions::users_groupes",
    ),
    (
        "eihwaz_sessions",
        "::runique::middleware::session::session_db",
    ),
];

fn related_module_tokens(table_name: &str) -> TokenStream2 {
    if let Some((_, module)) = FRAMEWORK_TABLES.iter().find(|(t, _)| *t == table_name) {
        let path: syn::Path = syn::parse_str(&format!("{}::Entity", module)).unwrap();
        quote! { #path }
    } else {
        let module = quote::format_ident!("{}", table_name);
        quote! { super::#module::Entity }
    }
}

/// The target entity of a relation, as a path expression.
pub(crate) fn target_entity(target: &syn::Ident) -> TokenStream2 {
    related_module_tokens(&to_snake_case(&target.to_string()))
}

/// The target entity's primary key column, read from the entity: a
/// `belongs_to` references it whatever it is named.
pub(crate) fn target_pk_column(target: &syn::Ident) -> TokenStream2 {
    let entity = target_entity(target);
    quote! {
        ::sea_orm::PrimaryKeyToColumn::into_column(
            <<#entity as ::sea_orm::EntityTrait>::PrimaryKey as ::sea_orm::Iterable>::iter()
                .next()
                .expect("an entity has a primary key"),
        )
    }
}

pub(crate) fn fk_action_tokens(action: FkAction) -> TokenStream2 {
    match action {
        FkAction::NoAction => quote! { ::sea_orm::sea_query::ForeignKeyAction::NoAction },
        FkAction::Cascade => quote! { ::sea_orm::sea_query::ForeignKeyAction::Cascade },
        FkAction::SetNull => quote! { ::sea_orm::sea_query::ForeignKeyAction::SetNull },
        FkAction::Restrict => quote! { ::sea_orm::sea_query::ForeignKeyAction::Restrict },
        FkAction::SetDefault => quote! { ::sea_orm::sea_query::ForeignKeyAction::SetDefault },
    }
}

/// `Relation` and its `RelationTrait`, written out rather than derived: a
/// `belongs_to` targets the related entity's real primary key and carries
/// its ON DELETE / ON UPDATE actions.
pub fn generate_relation_enum(model: &ModelInput) -> TokenStream2 {
    let variants: Vec<syn::Ident> = model.relations.iter().map(variant_name).collect();
    let defs: Vec<TokenStream2> = model.relations.iter().map(relation_def).collect();
    let related: Vec<TokenStream2> = model
        .relations
        .iter()
        .map(|r| generate_related_impl(r, &model.name))
        .collect();

    quote! {
        #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter)]
        pub enum Relation {
            #(#variants,)*
        }

        impl ::sea_orm::RelationTrait for Relation {
            fn def(&self) -> ::sea_orm::RelationDef {
                match *self {
                    #(Self::#variants => #defs,)*
                }
            }
        }

        #(#related)*
    }
}

fn variant_name(rel: &RelationDef) -> syn::Ident {
    match rel {
        RelationDef::BelongsTo { model, .. }
        | RelationDef::HasMany { model, .. }
        | RelationDef::HasOne { model, .. }
        | RelationDef::ManyToMany { model, .. } => ident_pascal(model),
    }
}

fn relation_def(rel: &RelationDef) -> TokenStream2 {
    match rel {
        RelationDef::BelongsTo {
            model: target,
            via,
            on_delete,
            on_update,
        } => {
            let entity = target_entity(target);
            let via_col = ident_pascal(via);
            let pk_column = target_pk_column(target);
            let on_delete = fk_action_tokens(*on_delete);
            let on_update = fk_action_tokens(*on_update);
            quote! {
                <Entity as ::sea_orm::EntityTrait>::belongs_to(#entity)
                    .from(Column::#via_col)
                    .to(#pk_column)
                    .on_delete(#on_delete)
                    .on_update(#on_update)
                    .into()
            }
        }
        RelationDef::HasMany { model: target, .. } => {
            let entity = target_entity(target);
            quote! { <Entity as ::sea_orm::EntityTrait>::has_many(#entity).into() }
        }
        RelationDef::HasOne { model: target, .. } => {
            let entity = target_entity(target);
            quote! { <Entity as ::sea_orm::EntityTrait>::has_one(#entity).into() }
        }
        RelationDef::ManyToMany { through, .. } => {
            let through_module = quote::format_ident!("{}", to_snake_case(&through.to_string()));
            quote! { <Entity as ::sea_orm::EntityTrait>::has_many(super::#through_module::Entity).into() }
        }
    }
}

fn generate_related_impl(rel: &RelationDef, self_model: &syn::Ident) -> TokenStream2 {
    match rel {
        RelationDef::BelongsTo { model: target, .. }
        | RelationDef::HasMany { model: target, .. }
        | RelationDef::HasOne { model: target, .. } => {
            let variant = ident_pascal(target);
            let entity_tokens = related_module_tokens(&to_snake_case(&target.to_string()));
            quote! {
                impl ::sea_orm::Related<#entity_tokens> for Entity {
                    fn to() -> ::sea_orm::RelationDef {
                        Relation::#variant.def()
                    }
                }
            }
        }

        RelationDef::ManyToMany {
            model: target,
            through,
            ..
        } => {
            let target_entity = related_module_tokens(&to_snake_case(&target.to_string()));
            let through_name = to_snake_case(&through.to_string());
            let through_module = quote::format_ident!("{}", through_name);
            let target_variant = ident_pascal(target);
            // The through entity's `belongs_to` pointing back to *this* model is
            // named after this model itself (`belongs_to`'s variant naming is
            // always `ident_pascal(target)`) — using the real model name directly
            // is exact, unlike guessing it from the `via_self` column string
            // (only coincidentally correct when the column happens to be named
            // `{self_model_snake_case}_id`).
            let via_self_variant = ident_pascal(self_model);

            quote! {
                impl ::sea_orm::Related<#target_entity> for Entity {
                    fn to() -> ::sea_orm::RelationDef {
                        super::#through_module::Relation::#target_variant.def()
                    }

                    fn via() -> Option<::sea_orm::RelationDef> {
                        Some(super::#through_module::Relation::#via_self_variant.def().rev())
                    }
                }
            }
        }
    }
}

fn ident_pascal(name: &syn::Ident) -> proc_macro2::Ident {
    quote::format_ident!("{}", pascal_case(&name.to_string()))
}

fn pascal_case(s: &str) -> String {
    s.split('_')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect()
}

fn to_snake_case(s: &str) -> String {
    // Already in snake_case (contains _ or all lowercase)
    if s.contains('_') || s.chars().all(|c| c.is_lowercase()) {
        return s.to_string();
    }
    // Otherwise PascalCase → snake_case conversion
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::to_snake_case;

    #[test]
    fn a_pascal_case_model_name_becomes_its_module_name() {
        assert_eq!(to_snake_case("Article"), "article");
        assert_eq!(to_snake_case("BlogPost"), "blog_post");
        assert_eq!(to_snake_case("blog_post"), "blog_post");
    }
}
