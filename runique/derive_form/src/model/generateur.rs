use crate::model::ast::*;
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn generate(model: &ModelInput) -> TokenStream2 {
    let schema = generate_schema(model);
    let sea_model = generate_sea_model(model);
    let relation_enum = generate_relation_enum();
    let active_model = generate_active_model();

    quote! {
        #schema
        #sea_model
        #relation_enum
        #active_model
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

pub fn generate_column(field: &FieldDef) -> TokenStream2 {
    let name = field.name.to_string();
    let ty = generate_field_type(&field.ty);
    let options: Vec<TokenStream2> = field.options.iter().map(generate_option).collect();

    quote! {
        .column(::runique::migration::ColumnDef::new(#name) #ty #(#options)*)
    }
}

fn generate_field_type(ty: &FieldType) -> TokenStream2 {
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
        FieldType::Enum(variants) => {
            let variant_strs: Vec<String> = variants.iter().map(|v| v.to_string()).collect();
            quote! { .enum_type("enum", vec![#(#variant_strs.to_string()),*]) }
        }
    }
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

            // note: les FK seront générées séparément
            let _ = (table, column, action);
            quote! {}
        }
    }
}
