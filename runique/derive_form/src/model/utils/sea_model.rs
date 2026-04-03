use crate::model::{FieldDef, FieldOption, FieldType, ModelInput, PkType};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── Struct SeaORM Model ───────────────────────────────────────

pub fn generate_sea_model(model: &ModelInput) -> TokenStream2 {
    let table = &model.table;
    let pk_field = generate_pk_field(model);
    let fields: Vec<TokenStream2> = model
        .fields
        .iter()
        .filter(|f| {
            !f.options
                .iter()
                .any(|o| matches!(o, FieldOption::AutoNow | FieldOption::AutoNowUpdate))
        })
        .map(generate_model_field)
        .collect();

    quote! {
        #[derive(Clone, Debug, PartialEq, ::sea_orm::DeriveEntityModel, ::serde::Serialize, ::serde::Deserialize)]
        #[sea_orm(table_name = #table)]
        pub struct Model {
            #pk_field
            #(#fields)*
        }
    }
}

fn generate_pk_field(model: &ModelInput) -> TokenStream2 {
    let name = &model.pk.name;
    let ty = match model.pk.ty {
        PkType::I32 => quote! { i32 },
        PkType::I64 => quote! { i64 },
        PkType::Uuid => quote! { ::sea_orm::prelude::Uuid },
    };
    quote! {
        #[sea_orm(primary_key)]
        pub #name: #ty,
    }
}

fn generate_model_field(field: &FieldDef) -> TokenStream2 {
    let name = &field.name;
    let nullable = field
        .options
        .iter()
        .any(|o| matches!(o, FieldOption::Nullable));
    let base_ty = field_type_to_rust(&field.ty);

    let ty = if nullable {
        quote! { Option<#base_ty> }
    } else {
        quote! { #base_ty }
    };

    quote! {
        pub #name: #ty,
    }
}

fn field_type_to_rust(ty: &FieldType) -> TokenStream2 {
    match ty {
        FieldType::String | FieldType::Text | FieldType::Char | FieldType::Varchar(_) => {
            quote! { String }
        }
        FieldType::I8 | FieldType::I16 | FieldType::I32 => quote! { i32 },
        FieldType::I64 => quote! { i64 },
        FieldType::U32 => quote! { u32 },
        FieldType::U64 => quote! { u64 },
        FieldType::F32 => quote! { f32 },
        FieldType::F64 => quote! { f64 },
        FieldType::Decimal(_) => quote! { ::rust_decimal::Decimal },
        FieldType::Bool => quote! { bool },
        FieldType::Date => quote! { ::chrono::NaiveDate },
        FieldType::Time => quote! { ::chrono::NaiveTime },
        FieldType::Datetime | FieldType::Timestamp | FieldType::TimestampTz => {
            quote! { ::chrono::NaiveDateTime }
        }
        FieldType::Uuid => quote! { ::sea_orm::prelude::Uuid },
        FieldType::Json | FieldType::JsonBinary => quote! { runique::serde_json::Value },
        FieldType::Binary(_) | FieldType::VarBinary(_) | FieldType::Blob => quote! { Vec<u8> },
        FieldType::Inet | FieldType::Cidr | FieldType::MacAddress | FieldType::Interval => {
            quote! { String }
        }
        FieldType::Enum(name) => quote! { #name },
    }
}
