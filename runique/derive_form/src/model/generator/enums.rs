//! Enum types declared in `enums:` — the Rust enums, and the label
//! resolver the admin display layer uses.
use crate::model::ast::*;
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates `apply_enum_labels(row)` — rewrites this model's enum columns of a
/// serialized JSON row to their display label (`Display` = the DSL label). Called
/// by the admin display layer (list/detail/delete), never on the edit-form data
/// path which needs the raw db value to pre-select the right choice. No-op when
/// the model declares no enum field.
pub(super) fn generate_enum_label_resolver(model: &ModelInput) -> TokenStream2 {
    let resolvers: Vec<TokenStream2> = model
        .fields
        .iter()
        .filter_map(|f| {
            let FieldType::Enum(enum_name) = f.column_type() else {
                return None;
            };
            let col = f.name.to_string();
            Some(quote! {
                {
                    let current = row
                        .get(#col)
                        .and_then(|v| v.as_str())
                        .map(::std::string::ToString::to_string);
                    if let Some(s) = current
                        && let Ok(e) = <#enum_name as ::std::str::FromStr>::from_str(&s)
                    {
                        row[#col] = ::runique::serde_json::Value::String(
                            ::std::string::ToString::to_string(&e),
                        );
                    }
                }
            })
        })
        .collect();

    quote! {
        pub fn apply_enum_labels(row: &mut ::runique::serde_json::Value) {
            let _ = &row;
            #(#resolvers)*
        }
    }
}

pub fn generate_enums(model: &ModelInput) -> TokenStream2 {
    generate_enum_defs(&model.enums)
}

/// Generates the Rust enum types (`DeriveActiveEnum` + `Display`/`FromStr`/`Default`)
/// for a list of `EnumDef`. Shared by `model!` and `extend!`.
pub fn generate_enum_defs(enums: &[EnumDef]) -> TokenStream2 {
    enums
        .iter()
        .map(|e| {
            let name = &e.name;
            let variant_names: Vec<&syn::Ident> = e.variants.iter().map(|v| &v.name).collect();
            let variant_name_strs: Vec<String> =
                variant_names.iter().map(|v| v.to_string()).collect();
            let first = &variant_names[0];

            match e.backing_type {
                EnumBackingType::I8
                | EnumBackingType::I16
                | EnumBackingType::I32
                | EnumBackingType::I64 => {
                    let (rs_type, db_type) = match e.backing_type {
                        EnumBackingType::I8 => (quote! { i8 }, "TinyInteger"),
                        EnumBackingType::I16 => (quote! { i16 }, "SmallInteger"),
                        EnumBackingType::I32 => (quote! { i32 }, "Integer"),
                        _ => (quote! { i64 }, "BigInteger"),
                    };
                    let rs_type_str = rs_type.to_string();
                    // Unsuffixed: the same literal fits whichever integer type the enum uses;
                    // runique_dsl already checked each value is within its range.
                    let db_values: Vec<proc_macro2::Literal> = e
                        .variants
                        .iter()
                        .filter_map(|v| v.int_value())
                        .map(proc_macro2::Literal::i64_unsuffixed)
                        .collect();
                    let display_values: Vec<String> =
                        e.variants.iter().map(|v| v.display_str()).collect();

                    quote! {
                        #[derive(
                            ::sea_orm::EnumIter, ::sea_orm::DeriveActiveEnum,
                            Clone, Debug, PartialEq,
                            ::serde::Serialize, ::serde::Deserialize,
                        )]
                        #[sea_orm(rs_type = #rs_type_str, db_type = #db_type)]
                        pub enum #name {
                            #(
                                #[sea_orm(num_value = #db_values)]
                                #variant_names,
                            )*
                        }

                        impl #name {
                            pub fn db_value(&self) -> #rs_type {
                                match self {
                                    #(#name::#variant_names => #db_values,)*
                                }
                            }

                            /// The value a form sends for this variant (its name).
                            pub fn form_value(&self) -> &'static str {
                                match self {
                                    #(#name::#variant_names => #variant_name_strs,)*
                                }
                            }
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
                                    #[serde(rename = #db_values)]
                                    #variant_names,
                                )*
                            }

                            impl #name {
                                pub fn db_value(&self) -> &'static str {
                                    match self {
                                        #(#name::#variant_names => #db_values,)*
                                    }
                                }

                                /// The value a form sends for this variant (its stored value).
                                pub fn form_value(&self) -> &'static str {
                                    self.db_value()
                                }
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
                                    #[serde(rename = #db_values)]
                                    #variant_names,
                                )*
                            }

                            impl #name {
                                pub fn db_value(&self) -> &'static str {
                                    match self {
                                        #(#name::#variant_names => #db_values,)*
                                    }
                                }

                                /// The value a form sends for this variant (its stored value).
                                pub fn form_value(&self) -> &'static str {
                                    self.db_value()
                                }
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
