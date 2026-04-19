use crate::model::ast::*;
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate(model: &ModelInput) -> TokenStream2 {
    let enums = generate_enums(model);
    let schema = generate_schema(model);
    let sea_model = generate_sea_model(model);
    let relation_enum = generate_relation_enum(model);
    let active_model = generate_active_model();
    let from_str_map: TokenStream2 = generate_from_str_map(model);
    let partial_update: TokenStream2 = generate_partial_update(model);
    let admin_form = generate_admin_form(model);

    quote! {
        #enums
        #schema
        #sea_model
        #relation_enum
        #active_model
        #from_str_map
        #partial_update
        #admin_form
    }
}

pub fn generate_enums(model: &ModelInput) -> TokenStream2 {
    model
        .enums
        .iter()
        .map(|e| {
            let name = &e.name;
            let variant_names: Vec<&syn::Ident> = e.variants.iter().map(|v| &v.name).collect();
            let variant_name_strs: Vec<String> =
                variant_names.iter().map(|v| v.to_string()).collect();
            let first = &variant_names[0];

            match e.backing_type {
                EnumBackingType::I32 => {
                    let db_values: Vec<i32> = e
                        .variants
                        .iter()
                        .map(|v| match &v.value {
                            Some(syn::Lit::Int(n)) => n.base10_parse::<i32>().unwrap_or(0),
                            _ => 0,
                        })
                        .collect();

                    quote! {
                        #[derive(
                            ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum,
                            Clone, Debug, PartialEq,
                            ::serde::Serialize, ::serde::Deserialize,
                        )]
                        #[sea_orm(rs_type = "i32", db_type = "Integer")]
                        pub enum #name {
                            #(
                                #[sea_orm(num_value = #db_values)]
                                #variant_names,
                            )*
                        }

                        impl ::std::default::Default for #name {
                            fn default() -> Self { #name::#first }
                        }

                        impl ::std::fmt::Display for #name {
                            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                                let s = match self {
                                    #(#name::#variant_names => #variant_name_strs,)*
                                };
                                f.write_str(s)
                            }
                        }

                        impl ::std::str::FromStr for #name {
                            type Err = ();
                            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                                #(
                                    if s == #variant_name_strs {
                                        return ::std::result::Result::Ok(#name::#variant_names);
                                    }
                                )*
                                ::std::result::Result::Err(())
                            }
                        }
                    }
                }

                EnumBackingType::I64 => {
                    let db_values: Vec<i64> = e
                        .variants
                        .iter()
                        .map(|v| match &v.value {
                            Some(syn::Lit::Int(n)) => n.base10_parse::<i64>().unwrap_or(0),
                            _ => 0,
                        })
                        .collect();

                    quote! {
                        #[derive(
                            ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum,
                            Clone, Debug, PartialEq,
                            ::serde::Serialize, ::serde::Deserialize,
                        )]
                        #[sea_orm(rs_type = "i64", db_type = "BigInteger")]
                        pub enum #name {
                            #(
                                #[sea_orm(num_value = #db_values)]
                                #variant_names,
                            )*
                        }

                        impl ::std::default::Default for #name {
                            fn default() -> Self { #name::#first }
                        }

                        impl ::std::fmt::Display for #name {
                            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                                let s = match self {
                                    #(#name::#variant_names => #variant_name_strs,)*
                                };
                                f.write_str(s)
                            }
                        }

                        impl ::std::str::FromStr for #name {
                            type Err = ();
                            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                                #(
                                    if s == #variant_name_strs {
                                        return ::std::result::Result::Ok(#name::#variant_names);
                                    }
                                )*
                                ::std::result::Result::Err(())
                            }
                        }
                    }
                }
                EnumBackingType::Auto => {
                    let db_values: Vec<String> = e.variants.iter().map(|v| v.db_str()).collect();
                    let display_values: Vec<String> =
                        e.variants.iter().map(|v| v.display_str()).collect();

                    let match_conditions: Vec<proc_macro2::TokenStream> = e
                        .variants
                        .iter()
                        .zip(variant_names.iter())
                        .map(|(v, vname)| {
                            let db_val = v.db_str();
                            let name_str = vname.to_string();
                            let display = v.display_str();
                            let mut conditions: Vec<proc_macro2::TokenStream> = vec![
                                quote! { s == #db_val },
                                quote! { s == #name_str },
                            ];
                            if display != db_val && display != name_str {
                                conditions.push(quote! { s == #display });
                            }
                            quote! {
                                if #(#conditions)||* {
                                    return ::std::result::Result::Ok(#name::#vname);
                                }
                            }
                        })
                        .collect();

                    let enum_name_str = e.name.to_string().to_ascii_lowercase();

                    let engine = DbEngine::detect();
                    if engine.is_unknown() {
                        let err_msg = format!(
                            "derive_form: unable to detect the database engine for enum `{}`. \
                            Add `DB_ENGINE=postgres` (or `mysql`/`sqlite`) to your `.env`.",
                            e.name
                        );
                        return quote! { ::std::compile_error!(#err_msg); };
                    }

                    if engine.is_postgres() {
                        quote! {
                            #[derive(
                                ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum,
                                Clone, Debug, PartialEq,
                                ::serde::Serialize, ::serde::Deserialize,
                            )]
                            #[sea_orm(rs_type = "String", db_type = "Enum", enum_name = #enum_name_str)]
                            pub enum #name {
                                #(
                                    #[sea_orm(string_value = #db_values)]
                                    #variant_names,
                                )*
                            }

                            impl ::std::default::Default for #name {
                                fn default() -> Self { #name::#first }
                            }

                            impl ::std::fmt::Display for #name {
                                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                                    let s = match self {
                                        #(#name::#variant_names => #display_values,)*
                                    };
                                    f.write_str(s)
                                }
                            }

                            impl ::std::str::FromStr for #name {
                                type Err = ();
                                fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                                    #(#match_conditions)*
                                    ::std::result::Result::Err(())
                                }
                            }
                        }
                    } else {
                        // MySQL / SQLite → VARCHAR (same as String)
                        quote! {
                            #[derive(
                                ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum,
                                Clone, Debug, PartialEq,
                                ::serde::Serialize, ::serde::Deserialize,
                            )]
                            #[sea_orm(rs_type = "String", db_type = "String(StringLen::None)")]
                            pub enum #name {
                                #(
                                    #[sea_orm(string_value = #db_values)]
                                    #variant_names,
                                )*
                            }

                            impl ::std::default::Default for #name {
                                fn default() -> Self { #name::#first }
                            }

                            impl ::std::fmt::Display for #name {
                                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                                    let s = match self {
                                        #(#name::#variant_names => #display_values,)*
                                    };
                                    f.write_str(s)
                                }
                            }

                            impl ::std::str::FromStr for #name {
                                type Err = ();
                                fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                                    #(#match_conditions)*
                                    ::std::result::Result::Err(())
                                }
                            }
                        }
                    }
                }
            }
        })
        .collect()
}

/// Generates `pub fn admin_from_form(data: &HashMap<String, String>, id: Option<PkType>) -> ActiveModel`
/// This function is used by the admin view to create/update a DB entry from form data (HashMap<String, String>).
pub fn generate_from_str_map(model: &ModelInput) -> TokenStream2 {
    let pk_name = &model.pk.name;

    // The PK value according to its type
    let pk_set = match model.pk.ty {
        PkType::I32 => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::NotSet,
            },
        },
        PkType::I64 => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::NotSet,
            },
        },
        PkType::Uuid => quote! {
            #pk_name: match __id {
                ::std::option::Option::Some(pk) => ::sea_orm::ActiveValue::Unchanged(pk),
                ::std::option::Option::None    => ::sea_orm::ActiveValue::Set(::sea_orm::prelude::Uuid::new_v4()),
            },
        },
    };

    // One assignment per field (auto_now/auto_now_update fields are excluded from Model → ignored)
    let field_assignments: Vec<TokenStream2> = model.fields.iter().filter_map(|field| {
        let fname = &field.name;
        let fname_str = fname.to_string();

        let is_auto_now = field.options.iter().any(|o| matches!(o, FieldOption::AutoNow));
        let is_auto_now_update = field.options.iter().any(|o| matches!(o, FieldOption::AutoNowUpdate));
        let is_nullable = field.options.iter().any(|o| matches!(o, FieldOption::Nullable));

        // These fields do not exist in the ActiveModel (filtered by generate_sea_model) → skipping them
        if is_auto_now || is_auto_now_update {
            return None;
        }

        let ts = match &field.ty {
            FieldType::Bool => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| {
                                let s = v.as_str();
                                s == "true" || s == "1" || s == "on"
                            })
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .map(|v| { let s = v.as_str(); s == "true" || s == "1" || s == "on" })
                                .unwrap_or(false)
                        ),
                    }
                }
            }
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::U32 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::I64 | FieldType::U64 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::F32 | FieldType::F64 => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| v.parse().ok()).unwrap_or_default()
                        ),
                    }
                }
            }
            FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz
            | FieldType::Date | FieldType::Time => {
                // Date/time fields without auto_now: left to Default (NotSet)
                return None;
            }
            FieldType::Uuid => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .and_then(|v| ::sea_orm::prelude::Uuid::parse_str(v).ok())
                                .unwrap_or_else(::sea_orm::prelude::Uuid::new_v4)
                        ),
                    }
                }
            }
            FieldType::Json | FieldType::JsonBinary => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .filter(|v| !v.is_empty())
                                .and_then(|v| ::runique::serde_json::from_str(v).ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .and_then(|v| ::runique::serde_json::from_str(v).ok())
                                .unwrap_or(::runique::serde_json::Value::Null)
                        ),
                    }
                }
            }
            FieldType::Enum(enum_name) => {
                if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .filter(|v| !v.is_empty())
                                .and_then(|v| v.parse::<#enum_name>().ok())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str)
                                .and_then(|v| v.parse::<#enum_name>().ok())
                                .unwrap_or_default()
                        ),
                    }
                }
            }
            // String, Text, Char, Varchar, Blob, Inet, Cidr, MacAddress, Interval, Binary, VarBinary
            _ => {
                let is_password = fname_str.contains("password");
                if is_password {
                    // Password fields: automatic hashing via developer's global config.
                    // If empty → NotSet (do not overwrite during an edit without a new password).
                    if is_nullable {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    Some(::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone()))
                                ),
                                None => ::sea_orm::ActiveValue::Set(None),
                            },
                        }
                    } else {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    ::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone())
                                ),
                                None => ::sea_orm::ActiveValue::NotSet,
                            },
                        }
                    }
                } else if is_nullable {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
                        ),
                    }
                } else {
                    quote! {
                        #fname: ::sea_orm::ActiveValue::Set(
                            __data.get(#fname_str).map(|v| v.trim().to_string()).unwrap_or_default()
                        ),
                    }
                }
            }
        };
        Some(ts)
    }).collect();

    // PK type for signature
    let pk_type = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };

    quote! {
        /// Builds an `ActiveModel` from a form data map (admin view).
        /// - `id = Some(pk)` → update (Unchanged on PK)
        /// - `id = None`     → creation (NotSet or Uuid::new_v4() depending on PK type)
        #[allow(clippy::needless_update)]
        pub fn admin_from_form(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::std::option::Option<#pk_type>,
        ) -> ActiveModel {
            ActiveModel {
                #pk_set
                #(#field_assignments)*
                ..::std::default::Default::default()
            }
        }
    }
}

/// Builds an `ActiveModel` for partial updates: only fields present in `data` are `Set`,
/// absent fields stay `NotSet` so SeaORM skips them entirely (no overwrite).
pub fn generate_partial_update(model: &ModelInput) -> TokenStream2 {
    let pk_name = &model.pk.name;

    let pk_set = match model.pk.ty {
        PkType::I32 | PkType::I64 => quote! {
            #pk_name: ::sea_orm::ActiveValue::Unchanged(__id),
        },
        PkType::Uuid => quote! {
            #pk_name: ::sea_orm::ActiveValue::Unchanged(__id),
        },
    };

    let pk_type = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };

    let field_assignments: Vec<TokenStream2> = model.fields.iter().filter_map(|field| {
        let fname = &field.name;
        let fname_str = fname.to_string();

        let is_auto_now = field.options.iter().any(|o| matches!(o, FieldOption::AutoNow));
        let is_auto_now_update = field.options.iter().any(|o| matches!(o, FieldOption::AutoNowUpdate));
        let is_nullable = field.options.iter().any(|o| matches!(o, FieldOption::Nullable));

        if is_auto_now || is_auto_now_update {
            return None;
        }

        let ts = match &field.ty {
            FieldType::Bool => {
                if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(Some({
                                let s = v.as_str(); s == "true" || s == "1" || s == "on"
                            })),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set({
                                let s = v.as_str(); s == "true" || s == "1" || s == "on"
                            }),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::U32
            | FieldType::I64 | FieldType::U64 | FieldType::F32 | FieldType::F64 => {
                if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(v.parse().ok()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(v.parse().unwrap_or_default()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
            FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz
            | FieldType::Date | FieldType::Time => return None,
            FieldType::Uuid => {
                if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(::sea_orm::prelude::Uuid::parse_str(v).ok()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(
                                ::sea_orm::prelude::Uuid::parse_str(v)
                                    .unwrap_or_else(|_| ::sea_orm::prelude::Uuid::new_v4())
                            ),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
            FieldType::Json | FieldType::JsonBinary => {
                if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str).filter(|v| !v.is_empty()) {
                            Some(v) => ::sea_orm::ActiveValue::Set(::runique::serde_json::from_str(v).ok()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str).filter(|v| !v.is_empty()) {
                            Some(v) => ::sea_orm::ActiveValue::Set(
                                ::runique::serde_json::from_str(v).unwrap_or(::runique::serde_json::Value::Null)
                            ),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
            FieldType::Enum(enum_name) => {
                if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str).filter(|v| !v.is_empty()) {
                            Some(v) => ::sea_orm::ActiveValue::Set(v.parse::<#enum_name>().ok()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str).filter(|v| !v.is_empty()) {
                            Some(v) => ::sea_orm::ActiveValue::Set(v.parse::<#enum_name>().unwrap_or_default()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
            _ => {
                let is_password = fname_str.contains("password");
                if is_password {
                    // Same behavior as admin_from_form: NotSet when empty
                    if is_nullable {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    Some(::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone()))
                                ),
                                None => ::sea_orm::ActiveValue::NotSet,
                            },
                        }
                    } else {
                        quote! {
                            #fname: match __data.get(#fname_str).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()) {
                                Some(v) => ::sea_orm::ActiveValue::Set(
                                    ::runique::utils::password::hash(&v).unwrap_or_else(|_| v.clone())
                                ),
                                None => ::sea_orm::ActiveValue::NotSet,
                            },
                        }
                    }
                } else if is_nullable {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(
                                Some(v.trim().to_string()).filter(|s| !s.is_empty())
                            ),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                } else {
                    quote! {
                        #fname: match __data.get(#fname_str) {
                            Some(v) => ::sea_orm::ActiveValue::Set(v.trim().to_string()),
                            None => ::sea_orm::ActiveValue::NotSet,
                        },
                    }
                }
            }
        };
        Some(ts)
    }).collect();

    quote! {
        /// Builds an `ActiveModel` for partial updates: only fields present in `data` are set.
        /// Fields absent from the map stay `NotSet` — SeaORM won't touch them.
        #[allow(clippy::needless_update)]
        pub fn admin_partial_update(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: #pk_type,
        ) -> ActiveModel {
            ActiveModel {
                #pk_set
                #(#field_assignments)*
                ..::std::default::Default::default()
            }
        }
    }
}

pub fn generate_pk(pk: &PkDef) -> TokenStream2 {
    let name = pk.name.to_string();
    match pk.ty {
        PkType::I32 => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).i32().auto_increment())
        },
        PkType::I64 => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).i64().auto_increment())
        },
        PkType::Uuid => quote! {
            .primary_key(::runique::migration::PrimaryKeyDef::new(#name).uuid())
        },
    }
}

pub fn generate_column(field: &FieldDef, enums: &[EnumDef]) -> TokenStream2 {
    let name = field.name.to_string();
    let ty = generate_field_type(&field.ty, enums);
    let options: Vec<TokenStream2> = field.options.iter().map(generate_option).collect();

    quote! {
        .column(::runique::migration::ColumnDef::new(#name) #ty #(#options)*)
    }
}

fn generate_field_type(ty: &FieldType, enums: &[EnumDef]) -> TokenStream2 {
    match ty {
        FieldType::String => quote! { .string() },
        FieldType::Text => quote! { .text() },
        FieldType::Char => quote! { .char() },
        FieldType::Varchar(n) => quote! { .varchar(#n) },
        FieldType::I8 => quote! { .tiny_integer() },
        FieldType::I16 => quote! { .small_integer() },
        FieldType::I32 => quote! { .integer() },
        FieldType::I64 => quote! { .big_integer() },
        FieldType::U32 => quote! { .unsigned() },
        FieldType::U64 => quote! { .big_unsigned() },
        FieldType::F32 => quote! { .float() },
        FieldType::F64 => quote! { .double() },
        FieldType::Decimal(None) => quote! { .decimal() },
        FieldType::Decimal(Some((p, s))) => quote! { .decimal_len(#p, #s) },
        FieldType::Bool => quote! { .boolean() },
        FieldType::Date => quote! { .date() },
        FieldType::Time => quote! { .time() },
        FieldType::Datetime => quote! { .datetime() },
        FieldType::Timestamp => quote! { .timestamp() },
        FieldType::TimestampTz => quote! { .timestamp_tz() },
        FieldType::Uuid => quote! { .uuid() },
        FieldType::Json => quote! { .json() },
        FieldType::JsonBinary => quote! { .json_binary() },
        FieldType::Binary(None) => quote! { .binary() },
        FieldType::Binary(Some(n)) => quote! { .binary_len(#n) },
        FieldType::VarBinary(n) => quote! { .var_binary(#n) },
        FieldType::Blob => quote! { .blob() },
        FieldType::Inet => quote! { .inet() },
        FieldType::Cidr => quote! { .cidr() },
        FieldType::MacAddress => quote! { .mac_address() },
        FieldType::Interval => quote! { .interval() },
        FieldType::Enum(enum_name) => {
            let enum_def = enums.iter().find(|e| e.name == *enum_name);
            if let Some(def) = enum_def {
                match &def.backing_type {
                    EnumBackingType::I32 => quote! { .integer() },
                    EnumBackingType::I64 => quote! { .big_integer() },
                    EnumBackingType::Auto => {
                        if DbEngine::detect().is_postgres() {
                            let name_str = def.name.to_string().to_ascii_lowercase();
                            let variants: Vec<String> = def
                                .variants
                                .iter()
                                .map(|v| match &v.value {
                                    Some(syn::Lit::Str(s)) => s.value(),
                                    Some(_) => v.name.to_string(),
                                    None => v.name.to_string(),
                                })
                                .collect();
                            quote! { .enum_type(#name_str, vec![#(#variants.to_string()),*]) }
                        } else {
                            quote! { .string() }
                        }
                    }
                }
            } else {
                quote! { .string() }
            }
        }
    }
}

/// Generates `{ModelName}AdminForm` — auto-generated form from model.
/// If `form_fields:` is declared in DSL, use explicit declarations.
/// Otherwise, infer widgets from SQL types (legacy behavior).
pub fn generate_admin_form(model: &ModelInput) -> TokenStream2 {
    let model_name = &model.name;
    let form_name = quote::format_ident!("{}AdminForm", model_name);

    let field_registrations: Vec<TokenStream2> = if !model.form_fields.is_empty() {
        model
            .form_fields
            .iter()
            .map(|ff| generate_form_field_decl(ff, model))
            .collect()
    } else {
        model.fields.iter().filter_map(|field| {
        let fname = &field.name;
        let fname_str = fname.to_string();

        let is_auto_now = field.options.iter().any(|o| matches!(o, FieldOption::AutoNow));
        let is_auto_now_update = field.options.iter().any(|o| matches!(o, FieldOption::AutoNowUpdate));
        let is_required = field.options.iter().any(|o| matches!(o, FieldOption::Required));
        let is_nullable = field.options.iter().any(|o| matches!(o, FieldOption::Nullable));

        if is_auto_now || is_auto_now_update {
            return None;
        }

        // Label: Use label("...") option if defined, otherwise generate from snake_case
        let label = if let Some(FieldOption::Label(lbl)) = field.options.iter().find(|o| matches!(o, FieldOption::Label(_))) {
            lbl.clone()
        } else {
            let s = fname_str.replace('_', " ");
            let mut chars = s.chars();
            match chars.next() {
                None => s,
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };

        let required_suffix = if is_required && !is_nullable {
            quote! { .required() }
        } else {
            quote! {}
        };

        // File field — priority over type inference
        if let Some(file_opt) = field.options.iter().find_map(|o| {
            if let FieldOption::File { kind, upload_to } = o { Some((kind, upload_to)) } else { None }
        }) {
            let (kind, upload_to) = file_opt;
            let file_constructor = match kind {
                FileKind::Image    => quote! { ::runique::forms::fields::FileField::image(#fname_str) },
                FileKind::Document => quote! { ::runique::forms::fields::FileField::document(#fname_str) },
                FileKind::Any      => quote! { ::runique::forms::fields::FileField::any(#fname_str) },
            };
            let upload_suffix = match upload_to {
                Some(path) => quote! { .upload_to(#path) },
                None       => quote! {},
            };
            // Search for max size option
            let size_suffix = if let Some(FieldOption::MaxSize(bytes)) = field.options.iter().find(|o| matches!(o, FieldOption::MaxSize(_))) {
                quote! { .max_size(#bytes) }
            } else {
                quote! {}
            };
            return Some(quote! {
                form.field(&#file_constructor.label(#label) #upload_suffix #size_suffix #required_suffix);
            });
        }

        let registration = match &field.ty {
            FieldType::Bool => quote! {
                form.field(&::runique::forms::fields::BooleanField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::I8 | FieldType::I16 | FieldType::I32 | FieldType::U32
            | FieldType::I64 | FieldType::U64 => quote! {
                form.field(&::runique::forms::fields::NumericField::integer(#fname_str).label(#label));
            },
            FieldType::F32 | FieldType::F64 => quote! {
                form.field(&::runique::forms::fields::NumericField::float(#fname_str).label(#label));
            },
            FieldType::Decimal(_) => quote! {
                form.field(&::runique::forms::fields::NumericField::decimal(#fname_str).label(#label));
            },
            FieldType::Date => quote! {
                form.field(&::runique::forms::fields::DateField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Time => quote! {
                form.field(&::runique::forms::fields::TimeField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz => quote! {
                form.field(&::runique::forms::fields::DateTimeField::new(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Json | FieldType::JsonBinary => quote! {
                form.field(&::runique::forms::fields::TextField::textarea(#fname_str).label(#label) #required_suffix);
            },
            FieldType::Enum(enum_name) => {
                let choices: Vec<proc_macro2::TokenStream> = model.enums.iter()
                    .find(|e| e.name == *enum_name)
                    .map(|e| e.variants.iter().map(|v| {
                        let db_val = v.db_str();
                        let display = v.display_str();
                        quote! { .add_choice(#db_val, #display) }
                    }).collect())
                    .unwrap_or_default();
                quote! {
                    form.field(
                        &::runique::forms::fields::ChoiceField::new(#fname_str)
                            .label(#label)
                            #(#choices)*
                            #required_suffix
                    );
                }
            },
            // String, Text, Char, Varchar, Uuid, Blob, Inet, Cidr, MacAddress, Interval, Binary, VarBinary
            _ => {
                let is_password = fname_str.contains("password");
                if is_password {
                    quote! {
                        form.field(&::runique::forms::fields::TextField::password(#fname_str).label(#label) #required_suffix);
                    }
                } else {
                    quote! {
                        form.field(&::runique::forms::fields::TextField::text(#fname_str).label(#label) #required_suffix);
                    }
                }
            }
        };

        Some(registration)
    }).collect()
    }; // end of if form_fields

    quote! {
        /// Stable alias used by the `runique start` daemon to reference this form.
        /// Always available via `{module}::AdminForm`.
        pub type AdminForm = #form_name;

        /// Admin form auto-generated from model.
        /// Covers all model fields except auto_now/auto_now_update fields.
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        pub struct #form_name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::field::RuniqueForm for #form_name {
            fn register_fields(form: &mut ::runique::forms::Forms) {
                #(#field_registrations)*
            }

            fn from_form(form: ::runique::forms::Forms) -> Self {
                Self { form }
            }

            fn get_form(&self) -> &::runique::forms::Forms {
                &self.form
            }

            fn get_form_mut(&mut self) -> &mut ::runique::forms::Forms {
                &mut self.form
            }
        }
    }
}

/// Generates `form.field(&...)` from a `FormFieldDecl` declaration (form_fields: block).
/// `model` is passed to allow resolution of enum variants for `Choice`/`Radio`.
fn generate_form_field_decl(ff: &FormFieldDecl, model: &ModelInput) -> TokenStream2 {
    // auto_now / auto_now_update fields: handled on DB side, no widget in the form.
    if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::AutoNow | FormFieldAttr::AutoNowUpdate))
    {
        return quote! {};
    }

    let name = &ff.name;
    let name_str = name.to_string();

    // Auto-generated label from snake_case name (same logic as SQL inference)
    let label = {
        let s = name_str.replace('_', " ");
        let mut chars = s.chars();
        match chars.next() {
            None => s,
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    };

    // Common attribute suffixes (order: constructor → label → specific attrs → required)
    let required_suffix = if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required))
    {
        quote! { .required() }
    } else {
        quote! {}
    };

    let field_expr: TokenStream2 = match &ff.kind {
        // ── Text fields ──────────────────────────────────────────────
        FormFieldKind::Text => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::text(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Email => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::email(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Password => {
            let no_hash = if ff.attrs.iter().any(|a| matches!(a, FormFieldAttr::NoHash)) {
                quote! { .no_hash() }
            } else {
                quote! {}
            };
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::password(#name_str).label(#label) #no_hash #extras #required_suffix }
        }
        FormFieldKind::Richtext => {
            let extras = text_attrs_tokens(&ff.attrs, true);
            quote! { ::runique::forms::fields::TextField::richtext(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Textarea => {
            let extras = text_attrs_tokens(&ff.attrs, true);
            quote! { ::runique::forms::fields::TextField::textarea(#name_str).label(#label) #extras #required_suffix }
        }
        FormFieldKind::Url => {
            let extras = text_attrs_tokens(&ff.attrs, false);
            quote! { ::runique::forms::fields::TextField::url(#name_str).label(#label) #extras #required_suffix }
        }

        // ── Numeric fields ─────────────────────────────────────────
        FormFieldKind::Int => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::integer(#name_str).label(#label) #extras }
        }
        FormFieldKind::Float => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::float(#name_str).label(#label) #extras }
        }
        FormFieldKind::Decimal => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::decimal(#name_str).label(#label) #extras }
        }
        FormFieldKind::Percent => {
            quote! { ::runique::forms::fields::NumericField::percent(#name_str).label(#label) }
        }

        // ── Bool ──────────────────────────────────────────────────────
        FormFieldKind::Bool => {
            let default_suffix = ff
                .attrs
                .iter()
                .find_map(|a| {
                    if let FormFieldAttr::Default(syn::Lit::Bool(b)) = a {
                        if b.value {
                            Some(quote! { .checked() })
                        } else {
                            Some(quote! { .unchecked() })
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_default();
            quote! { ::runique::forms::fields::BooleanField::new(#name_str).label(#label) #default_suffix #required_suffix }
        }

        // ── Date/Time ────────────────────────────────────────────────
        FormFieldKind::Date => {
            quote! { ::runique::forms::fields::DateField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Time => {
            quote! { ::runique::forms::fields::TimeField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Datetime => {
            quote! { ::runique::forms::fields::DateTimeField::new(#name_str).label(#label) #required_suffix }
        }

        // ── Files ──────────────────────────────────────────────────
        FormFieldKind::Image => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::image(#name_str).label(#label) #file_extras #required_suffix }
        }
        FormFieldKind::Document => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::document(#name_str).label(#label) #file_extras #required_suffix }
        }
        FormFieldKind::File => {
            let file_extras = file_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::FileField::any(#name_str).label(#label) #file_extras #required_suffix }
        }

        // ── Choice / Radio — resolution via EnumRef attr or fields: ─────
        FormFieldKind::Choice | FormFieldKind::Radio => {
            // Priority: Explicit EnumRef in attrs, otherwise lookup via fields:
            let enum_ident = ff.attrs.iter().find_map(|a| {
                if let FormFieldAttr::EnumRef(id) = a {
                    Some(id)
                } else {
                    None
                }
            });
            let enum_def = enum_ident
                .and_then(|id| model.enums.iter().find(|e| e.name == *id))
                .or_else(|| {
                    model
                        .fields
                        .iter()
                        .find(|f| f.name == ff.name)
                        .and_then(|field| {
                            if let FieldType::Enum(ename) = &field.ty {
                                model.enums.iter().find(|e| e.name == *ename)
                            } else {
                                None
                            }
                        })
                });

            let choices: Vec<TokenStream2> = enum_def
                .map(|e| {
                    e.variants
                        .iter()
                        .map(|v| {
                            let db_val = v.db_str();
                            let display = v.display_str();
                            quote! { .add_choice(#db_val, #display) }
                        })
                        .collect()
                })
                .unwrap_or_default();

            if matches!(ff.kind, FormFieldKind::Radio) {
                quote! { ::runique::forms::fields::RadioField::new(#name_str).label(#label) #(#choices)* #required_suffix }
            } else {
                quote! { ::runique::forms::fields::ChoiceField::new(#name_str).label(#label) #(#choices)* #required_suffix }
            }
        }

        // ── Special fields ───────────────────────────────────────────
        FormFieldKind::Color => {
            quote! { ::runique::forms::fields::ColorField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Slug => {
            quote! { ::runique::forms::fields::SlugField::new(#name_str).label(#label) }
        }
        FormFieldKind::Uuid => {
            quote! { ::runique::forms::fields::UUIDField::new(#name_str).label(#label) #required_suffix }
        }
        FormFieldKind::Json => {
            let rows_suffix = rows_token(&ff.attrs);
            quote! { ::runique::forms::fields::JSONField::new(#name_str).label(#label) #rows_suffix #required_suffix }
        }
        FormFieldKind::Ip => {
            quote! { ::runique::forms::fields::IPAddressField::new(#name_str).label(#label) #required_suffix }
        }

        FormFieldKind::Bigint => {
            let extras = numeric_attrs_tokens(&ff.attrs);
            quote! { ::runique::forms::fields::NumericField::integer(#name_str).label(#label) #extras }
        }
    };

    quote! { form.field(&#field_expr); }
}

/// Generates builder suffixes for text fields (max_length, min_length, rows).
fn text_attrs_tokens(attrs: &[FormFieldAttr], with_rows: bool) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::MaxLength(n) => ts.extend(quote! { .max_length(#n, "") }),
            FormFieldAttr::MinLength(n) => ts.extend(quote! { .min_length(#n, "") }),
            FormFieldAttr::Rows(n) if with_rows => {
                let n_usize = *n as usize;
                ts.extend(quote! { .rows(#n_usize) });
            }
            _ => {}
        }
    }
    ts
}

/// Generates builder suffixes for numeric fields (min, max, step).
fn numeric_attrs_tokens(attrs: &[FormFieldAttr]) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::Min(n) => {
                let v = *n as f64;
                ts.extend(quote! { .min(#v, "") });
            }
            FormFieldAttr::Max(n) => {
                let v = *n as f64;
                ts.extend(quote! { .max(#v, "") });
            }
            FormFieldAttr::MinF(n) => ts.extend(quote! { .min(#n, "") }),
            FormFieldAttr::MaxF(n) => ts.extend(quote! { .max(#n, "") }),
            FormFieldAttr::Step(n) => ts.extend(quote! { .step(#n) }),
            _ => {}
        }
    }
    ts
}

/// Generates builder suffixes for file fields (upload_to, max_size).
fn file_attrs_tokens(attrs: &[FormFieldAttr]) -> TokenStream2 {
    let mut ts = quote! {};
    for attr in attrs {
        match attr {
            FormFieldAttr::UploadTo(path) => ts.extend(quote! { .upload_to(#path) }),
            FormFieldAttr::MaxSize(n) => ts.extend(quote! { .max_size(#n) }),
            _ => {}
        }
    }
    ts
}

/// Generates the `.rows(n)` suffix if present.
fn rows_token(attrs: &[FormFieldAttr]) -> TokenStream2 {
    attrs
        .iter()
        .find_map(|a| {
            if let FormFieldAttr::Rows(n) = a {
                let n_usize = *n as usize;
                Some(quote! { .rows(#n_usize) })
            } else {
                None
            }
        })
        .unwrap_or_default()
}

fn generate_option(opt: &FieldOption) -> TokenStream2 {
    match opt {
        FieldOption::Required => quote! { .required() },
        FieldOption::Nullable => quote! { .nullable() },
        FieldOption::Unique => quote! { .unique() },
        FieldOption::Index => quote! {},
        FieldOption::AutoNow => quote! { .auto_now() },
        FieldOption::AutoNowUpdate => quote! { .auto_now_update() },
        FieldOption::Readonly => quote! { .ignore() },
        FieldOption::MaxLen(n) => quote! { .max_len(#n) },
        FieldOption::MinLen(n) => quote! { .min_len(#n) },
        FieldOption::Max(n) => quote! { .max_i64(#n) },
        FieldOption::Min(n) => quote! { .min_i64(#n) },
        FieldOption::MaxF(n) => quote! { .max_f64(#n) },
        FieldOption::MinF(n) => quote! { .min_f64(#n) },
        FieldOption::SelectAs(s) => quote! { .select_as(#s) },
        FieldOption::Default(lit) => quote! { .default(sea_query::Value::from(#lit)) },
        FieldOption::Label(_) | FieldOption::Help(_) => quote! {},
        FieldOption::File { .. } => quote! {},
        FieldOption::MaxSize(n) => quote! { .max_size(#n) },
        FieldOption::Fk(fk) => {
            let table = fk.table.to_string();
            let column = fk.column.to_string();
            let action = match fk.action {
                FkAction::Cascade => quote! { ::sea_orm::sea_query::ForeignKeyAction::Cascade },
                FkAction::SetNull => quote! { ::sea_orm::sea_query::ForeignKeyAction::SetNull },
                FkAction::Restrict => quote! { ::sea_orm::sea_query::ForeignKeyAction::Restrict },
                FkAction::SetDefault => {
                    quote! { ::sea_orm::sea_query::ForeignKeyAction::SetDefault }
                }
            };

            // note: FKs will be generated separately
            let _ = (table, column, action);
            quote! {}
        }
    }
}
