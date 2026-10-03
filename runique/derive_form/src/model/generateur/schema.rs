//! `schema()`: primary key, columns and their options, unique fields.
use crate::model::ast::*;
use crate::model::utils::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

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
    let ty = generate_field_type(&field.column_type(), enums);
    let options: Vec<TokenStream2> = field.options.iter().map(generate_option).collect();
    let kind = kind_tokens(field.kind);
    // The form needs the choices on every engine; only Postgres also gets
    // them in the SQL type (`enum_type` above).
    let choices = match &field.column_type() {
        FieldType::Enum(enum_name) => enums
            .iter()
            .find(|e| e.name == *enum_name)
            .map(|def| {
                let values: Vec<String> = def.variants.iter().map(|v| v.db_str()).collect();
                let labels: Vec<String> = def.variants.iter().map(|v| v.display_str()).collect();
                quote! { .choices(vec![#((#values.to_string(), #labels.to_string())),*]) }
            })
            .unwrap_or_default(),
        _ => quote! {},
    };

    quote! {
        .column(::runique::migration::ColumnDef::new(#name) #ty #kind #choices #(#options)*)
    }
}

/// `.kind(FormFieldKind::…)`: the DSL type travels with the column, so the
/// form rebuilt from the schema gets that type's field.
pub(crate) fn kind_tokens(kind: FormFieldKind) -> TokenStream2 {
    let variant = quote::format_ident!("{}", format!("{kind:?}"));
    quote! { .kind(::runique::runique_dsl::ast::FormFieldKind::#variant) }
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

fn generate_option(opt: &FieldOption) -> TokenStream2 {
    match opt {
        FieldOption::Required => quote! { .required() },
        FieldOption::Nullable => quote! { .nullable() },
        FieldOption::Unique => quote! { .unique() },
        FieldOption::AutoNow => quote! { .auto_now() },
        FieldOption::AutoNowUpdate => quote! { .auto_now_update() },
        FieldOption::Readonly => quote! { .ignore() },
        FieldOption::MaxLen(n) => quote! { .max_len(#n) },
        FieldOption::MinLen(n) => quote! { .min_len(#n) },
        FieldOption::Max(n) => quote! { .max_i64(#n) },
        FieldOption::Min(n) => quote! { .min_i64(#n) },
        FieldOption::MaxF(n) => quote! { .max_f64(#n) },
        FieldOption::MinF(n) => quote! { .min_f64(#n) },
        FieldOption::Default(lit) => quote! { .default(sea_query::Value::from(#lit)) },
        FieldOption::Label(label) => quote! { .label(#label) },
        FieldOption::File { kind, .. } => {
            let kind_tok = match kind {
                FileKind::Image => quote! { ::runique::migration::FileKind::Image },
                FileKind::Document => quote! { ::runique::migration::FileKind::Document },
                FileKind::Any => quote! { ::runique::migration::FileKind::Any },
            };
            quote! { .file(#kind_tok) }
        }
        FieldOption::MaxSize(n) => {
            quote! { .max_size_bytes(#n) }
        }
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

pub fn generate_unique_fields(model: &ModelInput) -> TokenStream2 {
    let names: Vec<String> = model
        .fields
        .iter()
        .filter(|f| f.options.iter().any(|o| matches!(o, FieldOption::Unique)))
        .map(|f| f.name.to_string())
        .collect();
    quote! {
        pub const UNIQUE_FIELDS: &[&str] = &[#(#names),*];
    }
}
