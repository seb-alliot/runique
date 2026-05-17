use crate::model::{FieldOption, FkDef, ModelInput, RelationDef};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Tables provided by the framework — their SeaORM entities are in `runique`, not in `super::`.
const FRAMEWORK_TABLES: &[(&str, &str)] = &[
    ("eihwaz_users", "::runique::auth::user"),
    ("eihwaz_groupes", "::runique::admin::permissions::groupe"),
    (
        "eihwaz_groupes_droits",
        "::runique::admin::permissions::groupes_droits",
    ),
    (
        "eihwaz_users_groupes",
        "::runique::admin::permissions::users_groupes",
    ),
    (
        "eihwaz_sessions",
        "::runique::middleware::session::session_db",
    ),
];

fn entity_path(table_name: &str) -> String {
    if let Some((_, module)) = FRAMEWORK_TABLES.iter().find(|(t, _)| *t == table_name) {
        format!("{}::Entity", module)
    } else {
        format!("super::{}::Entity", table_name)
    }
}

fn related_module_tokens(table_name: &str) -> TokenStream2 {
    if let Some((_, module)) = FRAMEWORK_TABLES.iter().find(|(t, _)| *t == table_name) {
        let path: syn::Path = syn::parse_str(&format!("{}::Entity", module)).unwrap();
        quote! { #path }
    } else {
        let module = quote::format_ident!("{}", table_name);
        quote! { super::#module::Entity }
    }
}

pub fn generate_relation_enum(model: &ModelInput) -> TokenStream2 {
    // FK fields already covered by an explicit belongs_to (matched by via field name)
    let explicit_vias: std::collections::HashSet<String> = model
        .relations
        .iter()
        .filter_map(|r| {
            if let RelationDef::BelongsTo { via, .. } = r {
                Some(via.to_string())
            } else {
                None
            }
        })
        .collect();

    // FK fields not already covered → auto-generate BelongsTo for pivot tables
    let auto_fks: Vec<(&syn::Ident, &FkDef)> = model
        .fields
        .iter()
        .filter_map(|f| {
            if explicit_vias.contains(&f.name.to_string()) {
                return None;
            }
            f.options.iter().find_map(|opt| {
                if let FieldOption::Fk(fk) = opt {
                    Some((&f.name, fk))
                } else {
                    None
                }
            })
        })
        .collect();

    let explicit_variants: Vec<TokenStream2> =
        model.relations.iter().map(generate_variant).collect();
    let explicit_related: Vec<TokenStream2> =
        model.relations.iter().map(generate_related_impl).collect();

    let fk_variants: Vec<TokenStream2> = auto_fks
        .iter()
        .map(|(name, fk)| generate_fk_variant(name, fk))
        .collect();
    let fk_related: Vec<TokenStream2> = auto_fks
        .iter()
        .map(|(name, fk)| generate_fk_related_impl(name, fk))
        .collect();

    if explicit_variants.is_empty() && fk_variants.is_empty() {
        return quote! {
            #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
            pub enum Relation {}
        };
    }

    quote! {
        #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
        pub enum Relation {
            #(#explicit_variants)*
            #(#fk_variants)*
        }

        #(#explicit_related)*
        #(#fk_related)*
    }
}

fn generate_fk_variant(field_name: &syn::Ident, fk_def: &FkDef) -> TokenStream2 {
    let field_str = field_name.to_string();
    let variant_base = field_str.strip_suffix("_id").unwrap_or(&field_str);
    let variant = quote::format_ident!("{}", pascal_case(variant_base));
    let module = table_to_module(&fk_def.table.to_string());
    let mod_path = entity_path(&module);
    let to_col = pascal_case(&fk_def.column.to_string());
    let to_path = format!(
        "{}::Column::{}",
        &mod_path[..mod_path.len() - "::Entity".len()],
        to_col
    );
    let via_col = format!("Column::{}", pascal_case(&field_str));
    quote! {
        #[sea_orm(
            belongs_to = #mod_path,
            from = #via_col,
            to = #to_path
        )]
        #variant,
    }
}

fn generate_fk_related_impl(field_name: &syn::Ident, fk_def: &FkDef) -> TokenStream2 {
    let field_str = field_name.to_string();
    let variant_base = field_str.strip_suffix("_id").unwrap_or(&field_str);
    let variant = quote::format_ident!("{}", pascal_case(variant_base));
    let module = table_to_module(&fk_def.table.to_string());
    let entity_tokens = related_module_tokens(&module);
    quote! {
        impl ::sea_orm::Related<#entity_tokens> for Entity {
            fn to() -> ::sea_orm::RelationDef {
                Relation::#variant.def()
            }
        }
    }
}

fn generate_variant(rel: &RelationDef) -> TokenStream2 {
    match rel {
        RelationDef::BelongsTo { model: target, via } => {
            let variant = ident_pascal(target);
            let via_col = format!("Column::{}", ident_pascal(via));
            let table = to_snake_case(&target.to_string());
            let mod_path = entity_path(&table);
            let to_path = format!(
                "{}::Column::Id",
                &mod_path[..mod_path.len() - "::Entity".len()]
            );
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
            let mod_path = entity_path(&to_snake_case(&target.to_string()));
            quote! {
                #[sea_orm(has_many = #mod_path)]
                #variant,
            }
        }

        RelationDef::HasOne { model: target, .. } => {
            let variant = ident_pascal(target);
            let mod_path = entity_path(&to_snake_case(&target.to_string()));
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
            let through_mod = to_snake_case(&through.to_string());
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
            via_self,
        } => {
            let target_entity = related_module_tokens(&to_snake_case(&target.to_string()));
            let through_name = to_snake_case(&through.to_string());
            let through_module = quote::format_ident!("{}", through_name);
            let target_variant = ident_pascal(target);
            // via_self is a FK column name (e.g. "menu_id") — strip "_id" to get the relation variant
            let via_self_str = via_self.to_string();
            let via_self_model = via_self_str.strip_suffix("_id").unwrap_or(&via_self_str);
            let via_self_variant = quote::format_ident!("{}", pascal_case(via_self_model));

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

/// Convert a SQL table name (often plural) to the Rust module name (singular model snake_case).
/// Framework tables are left as-is since they're matched by FRAMEWORK_TABLES.
/// Simple heuristic: strip trailing `s` unless it's `ss` or the table doesn't end with `s`.
fn table_to_module(table_name: &str) -> String {
    if FRAMEWORK_TABLES.iter().any(|(t, _)| *t == table_name) {
        return table_name.to_string();
    }
    if table_name.ends_with('s') && !table_name.ends_with("ss") {
        table_name[..table_name.len() - 1].to_string()
    } else {
        table_name.to_string()
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
