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

/// The column's type, from the same mapping as the entity's `column_type`
/// and the migration: storage SeaORM and the three engines understand; what
/// the field means (`char`, `ip`…) travels in `.kind(..)` for validation.
fn generate_field_type(ty: &FieldType, enums: &[EnumDef]) -> TokenStream2 {
    if let Some(column_type) = ty.sea_orm_column_type() {
        let column_type: TokenStream2 = column_type
            .parse()
            .expect("`sea_orm_column_type` is valid Rust");
        return quote! { .column_type(::runique::sea_orm::sea_query::ColumnType::#column_type) };
    }
    let FieldType::Enum(enum_name) = ty else {
        unreachable!("every type but an enum has a `sea_orm_column_type`");
    };
    let Some(def) = enums.iter().find(|e| e.name == *enum_name) else {
        return quote! { .string() };
    };
    match &def.backing_type {
        EnumBackingType::I8 => quote! { .tiny_integer() },
        EnumBackingType::I16 => quote! { .small_integer() },
        EnumBackingType::I32 => quote! { .integer() },
        EnumBackingType::I64 => quote! { .big_integer() },
        EnumBackingType::Auto if DbEngine::detect().is_postgres() => {
            let name_str = def.name.to_string().to_ascii_lowercase();
            let variants: Vec<String> = def
                .variants
                .iter()
                .map(|v| match &v.value {
                    Some(syn::Lit::Str(s)) => s.value(),
                    _ => v.name.to_string(),
                })
                .collect();
            quote! { .enum_type(#name_str, vec![#(#variants.to_string()),*]) }
        }
        EnumBackingType::Auto => quote! { .string() },
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
        // Already in the column's type: `.decimal_len(max_digits, decimal_places)`.
        FieldOption::DecimalLen { .. } => quote! {},
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
