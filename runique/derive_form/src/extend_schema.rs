use crate::model::ast::{FormFieldAttr, FormFieldDecl, FormFieldKind};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, LitStr, Token, braced, parse::ParseStream};

// ── DSL ──────────────────────────────────────────────────────────────────────

pub(crate) struct ExtendDsl {
    pub table: String,
    pub fields: Vec<FormFieldDecl>,
}

impl syn::parse::Parse for ExtendDsl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        let kw: Ident = input.parse()?;
        if kw != "fields" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'fields'"));
        }
        input.parse::<Token![:]>()?;
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(FormFieldDecl::parse(&fields_content)?);
        }
        let _ = input.parse::<Token![,]>();

        Ok(ExtendDsl {
            table: table.value(),
            fields,
        })
    }
}

// ── Code generation ──────────────────────────────────────────────────────────

fn field_to_coldef(ff: &FormFieldDecl) -> TokenStream2 {
    let name = ff.name.to_string();
    let is_required = ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Required));
    let nullable = if is_required {
        quote! {}
    } else {
        quote! { .nullable() }
    };
    let max_length = ff.attrs.iter().find_map(|a| {
        if let FormFieldAttr::MaxLength(n) = a {
            Some(*n)
        } else {
            None
        }
    });

    let type_call = match &ff.kind {
        FormFieldKind::Textarea | FormFieldKind::Richtext => quote! { .text() },
        FormFieldKind::Int => quote! { .integer() },
        FormFieldKind::Bigint => quote! { .big_integer() },
        FormFieldKind::Float | FormFieldKind::Percent => quote! { .double() },
        FormFieldKind::Decimal => quote! { .decimal() },
        FormFieldKind::Bool => quote! { .boolean() },
        FormFieldKind::Date => quote! { .date() },
        FormFieldKind::Time => quote! { .time() },
        FormFieldKind::Datetime => quote! { .datetime() },
        FormFieldKind::Uuid => quote! { .uuid() },
        FormFieldKind::Json => quote! { .json() },
        // Text variants + files + choice → varchar(n) or string()
        _ => {
            if let Some(n) = max_length {
                quote! { .varchar(#n) }
            } else {
                quote! { .string() }
            }
        }
    };

    quote! {
        .column(
            ::runique::migration::column::ColumnDef::new(#name)
                #type_call
                #nullable
        )
    }
}

/// Generates `pub fn schema() -> ModelSchema { ... }` from the parsed DSL.
pub(crate) fn generate_schema_fn(dsl: &ExtendDsl) -> TokenStream2 {
    let table = &dsl.table;
    let col_defs: Vec<TokenStream2> = dsl.fields.iter().map(field_to_coldef).collect();

    quote! {
        pub fn schema() -> ::runique::migration::schema::ModelSchema {
            ::runique::migration::schema::ModelSchema::new(#table)
                .table_name(#table)
                #(#col_defs)*
        }
    }
}
