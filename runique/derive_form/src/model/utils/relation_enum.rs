use crate::model::{ModelInput, RelationDef};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate_relation_enum(model: &ModelInput) -> TokenStream2 {
    if model.relations.is_empty() {
        return quote! {
            #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
            pub enum Relation {}
        };
    }

    let variants: Vec<TokenStream2> = model.relations.iter().map(generate_variant).collect();

    let related_impls: Vec<TokenStream2> =
        model.relations.iter().map(generate_related_impl).collect();

    quote! {
        #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
        pub enum Relation {
            #(#variants)*
        }

        #(#related_impls)*
    }
}

fn generate_variant(rel: &RelationDef) -> TokenStream2 {
    match rel {
        RelationDef::BelongsTo { model: target, via } => {
            let variant = ident_pascal(target);
            let via_col = format!("Column::{}", ident_pascal(via));
            let mod_path = format!("super::{}::Entity", target.to_string().to_lowercase());
            let to_path = format!("super::{}::Column::Id", to_snake_case(&target.to_string()));
            quote! {
                #[sea_orm(
                    belongs_to = #mod_path,
                    from = #via_col,
                    to = #to_path
                )]
                #variant,
            }
        }

        RelationDef::HasMany { model: target, .. } => {
            let variant = ident_pascal(target);
            let mod_path = format!("super::{}::Entity", target.to_string().to_lowercase());
            quote! {
                #[sea_orm(has_many = #mod_path)]
                #variant,
            }
        }

        RelationDef::HasOne { model: target, .. } => {
            let variant = ident_pascal(target);
            let mod_path = format!("super::{}::Entity", target.to_string().to_lowercase());
            quote! {
                #[sea_orm(has_one = #mod_path)]
                #variant,
            }
        }

        RelationDef::ManyToMany {
            model: target,
            through,
            ..
        } => {
            let variant = ident_pascal(target);
            let through_mod = through.to_string().to_lowercase();
            let mod_path = format!("super::{}::Entity", through_mod);
            quote! {
                #[sea_orm(has_many = #mod_path)]
                #variant,
            }
        }
    }
}

fn generate_related_impl(rel: &RelationDef) -> TokenStream2 {
    match rel {
        RelationDef::BelongsTo { model: target, .. }
        | RelationDef::HasMany { model: target, .. }
        | RelationDef::HasOne { model: target, .. } => {
            let variant = ident_pascal(target);
            let module = quote::format_ident!("{}", target.to_string().to_lowercase());
            quote! {
                impl ::sea_orm::Related<super::#module::Entity> for Entity {
                    fn to() -> ::sea_orm::RelationDef {
                        Relation::#variant.def()
                    }
                }
            }
        }

        RelationDef::ManyToMany {
            model: target,
            through,
            via_self,
        } => {
            let target_module = quote::format_ident!("{}", target.to_string().to_lowercase());
            let through_module = quote::format_ident!("{}", through.to_string().to_lowercase());
            let target_variant = ident_pascal(target);
            let via_self_variant = ident_pascal(via_self);

            quote! {
                impl ::sea_orm::Related<super::#target_module::Entity> for Entity {
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
    // Déjà en snake_case (contient _ ou tout en minuscules)
    if s.contains('_') || s.chars().all(|c| c.is_lowercase()) {
        return s.to_string();
    }
    // Sinon conversion PascalCase → snake_case
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}
