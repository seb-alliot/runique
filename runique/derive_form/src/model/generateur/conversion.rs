//! Form data → `ActiveModel`: `admin_from_form`, `admin_partial_update`, and
//! the per-field conversion `extend!{}` shares.
use crate::model::ast::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

/// Generates `pub fn admin_from_form(data, id) -> Result<ActiveModel, FormDataError>`
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
                ::std::option::Option::None    => ::sea_orm::ActiveValue::Set(::sea_orm::prelude::Uuid::now_v7()),
            },
        },
    };

    // One assignment per field (auto_now/auto_now_update fields are excluded from Model → ignored)
    let field_assignments: Vec<TokenStream2> =
        model.fields.iter().filter_map(field_from_form).collect();

    // PK type for signature
    let pk_type = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };

    quote! {
        /// Builds an `ActiveModel` from a form data map (admin view).
        /// - `id = Some(pk)` → update (Unchanged on PK)
        /// - `id = None`     → creation (NotSet, or Uuid::now_v7() when the PK type is Uuid)
        #[allow(clippy::needless_update)]
        pub fn admin_from_form(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: ::std::option::Option<#pk_type>,
        ) -> ::std::result::Result<ActiveModel, ::runique::forms::FormDataError> {
            ::std::result::Result::Ok(ActiveModel {
                #pk_set
                #(#field_assignments)*
                ..::std::default::Default::default()
            })
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

    let field_assignments: Vec<TokenStream2> =
        model.fields.iter().filter_map(field_from_partial).collect();

    quote! {
        /// Builds an `ActiveModel` for partial updates: only fields present in `data` are set.
        /// Fields absent from the map stay `NotSet` — SeaORM won't touch them.
        #[allow(clippy::needless_update)]
        pub fn admin_partial_update(
            __data: &::std::collections::HashMap<::std::string::String, ::std::string::String>,
            __id: #pk_type,
        ) -> ::std::result::Result<ActiveModel, ::runique::forms::FormDataError> {
            ::std::result::Result::Ok(ActiveModel {
                #pk_set
                #(#field_assignments)*
                ..::std::default::Default::default()
            })
        }
    }
}

/// How a field's raw string is read into its Rust type: a `&str -> Option<T>`
/// closure handed to `runique::forms::form_data::read`.
fn parser(ty: &FieldType) -> TokenStream2 {
    match ty {
        FieldType::Bool => quote! { |v: &str| Some(matches!(v, "true" | "1" | "on")) },
        FieldType::I8
        | FieldType::I16
        | FieldType::I32
        | FieldType::I64
        | FieldType::U32
        | FieldType::U64 => {
            let rust = crate::model::utils::sea_model::field_type_to_rust(ty);
            quote! { |v: &str| v.parse::<#rust>().ok() }
        }
        FieldType::F32 | FieldType::F64 => {
            let rust = crate::model::utils::sea_model::field_type_to_rust(ty);
            quote! { |v: &str| v.replace(',', ".").parse::<#rust>().ok().filter(|x| x.is_finite()) }
        }
        FieldType::Decimal(_) => quote! {
            |v: &str| ::runique::sea_orm::prelude::Decimal::from_str_exact(&v.replace(',', ".")).ok()
        },
        FieldType::Date => {
            quote! { |v: &str| ::chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").ok() }
        }
        FieldType::Time => quote! {
            |v: &str| ::chrono::NaiveTime::parse_from_str(v, "%H:%M:%S")
                .or_else(|_| ::chrono::NaiveTime::parse_from_str(v, "%H:%M"))
                .ok()
        },
        FieldType::Datetime | FieldType::Timestamp => {
            quote! { |v: &str| ::runique::forms::fields::parse_datetime_local(v) }
        }
        FieldType::TimestampTz => {
            quote! { |v: &str| ::runique::forms::fields::parse_utc_datetime(v) }
        }
        FieldType::Uuid => quote! { |v: &str| ::sea_orm::prelude::Uuid::parse_str(v).ok() },
        FieldType::Json | FieldType::JsonBinary => quote! {
            |v: &str| ::runique::serde_json::from_str::<::runique::serde_json::Value>(v).ok()
        },
        FieldType::Enum(name) => quote! { |v: &str| v.parse::<#name>().ok() },
        FieldType::Binary(_) | FieldType::VarBinary(_) | FieldType::Blob => {
            quote! { |v: &str| ::runique::forms::fields::decode_binary(v) }
        }
        FieldType::String
        | FieldType::Text
        | FieldType::Char
        | FieldType::Varchar(_)
        | FieldType::Inet
        | FieldType::Cidr
        | FieldType::MacAddress
        | FieldType::Interval => quote! { |v: &str| Some(v.to_string()) },
    }
}

/// The value expression for one field in `admin_from_form`: read from the
/// form data, an error if it's there but unreadable. `None` for columns the
/// form never sets (`auto_now`, `auto_now_update`).
fn field_value(field: &FieldDef) -> Option<TokenStream2> {
    let has = |wanted: fn(&FieldOption) -> bool| field.options.iter().any(wanted);
    if has(|o| matches!(o, FieldOption::AutoNow | FieldOption::AutoNowUpdate)) {
        return None;
    }
    let nullable = has(|o| matches!(o, FieldOption::Nullable));
    let name = field.name.to_string();
    let ty = field.column_type();
    let parse = parser(&ty);

    // Nothing submitted for these leaves the stored value as it is: a password
    // (no new one typed), a file or bytes (no new upload).
    let keep_when_absent = matches!(
        field.kind,
        FormFieldKind::Password
            | FormFieldKind::Image
            | FormFieldKind::Document
            | FormFieldKind::File
    ) || matches!(
        ty,
        FieldType::Binary(_) | FieldType::VarBinary(_) | FieldType::Blob
    );

    let value = if field.kind == FormFieldKind::Password {
        quote! { ::runique::forms::form_data::password(&v, #name)? }
    } else {
        quote! { v }
    };
    let set = if nullable {
        quote! { ::sea_orm::ActiveValue::Set(Some(#value)) }
    } else {
        quote! { ::sea_orm::ActiveValue::Set(#value) }
    };
    let absent = if keep_when_absent {
        quote! { ::sea_orm::ActiveValue::NotSet }
    } else if nullable {
        quote! { ::sea_orm::ActiveValue::Set(None) }
    } else {
        match ty {
            // An unchecked box isn't sent at all.
            FieldType::Bool => quote! { ::sea_orm::ActiveValue::Set(false) },
            FieldType::String
            | FieldType::Text
            | FieldType::Char
            | FieldType::Varchar(_)
            | FieldType::Inet
            | FieldType::Cidr
            | FieldType::MacAddress
            | FieldType::Interval => {
                quote! { ::sea_orm::ActiveValue::Set(::std::string::String::new()) }
            }
            _ => quote! {
                return ::std::result::Result::Err(
                    ::runique::forms::FormDataError::Required(#name.to_string()),
                )
            },
        }
    };

    Some(quote! {
        match ::runique::forms::form_data::read(__data, #name, #parse)? {
            Some(v) => #set,
            None => #absent,
        }
    })
}

/// `field: value,` read from the form data for `admin_from_form`; `None` for
/// columns the form never sets (`auto_now`, `auto_now_update`).
pub(crate) fn field_from_form(field: &FieldDef) -> Option<TokenStream2> {
    let fname = &field.name;
    let value = field_value(field)?;
    Some(quote! { #fname: #value, })
}

/// Same as [`field_from_form`] for `admin_partial_update`: a key absent from
/// the data leaves the column untouched (`NotSet`).
pub(crate) fn field_from_partial(field: &FieldDef) -> Option<TokenStream2> {
    let fname = &field.name;
    let name = fname.to_string();
    let value = field_value(field)?;
    Some(quote! {
        #fname: if __data.contains_key(#name) { #value } else { ::sea_orm::ActiveValue::NotSet },
    })
}
