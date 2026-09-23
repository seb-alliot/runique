//! `DslModel` — the top-level `model!{ Name, table: "...", pk: id => Pk, { ... } }`
//! invocation. `Parse` is split into one function per concern (header, `enums:`
//! block, `relations:`/`meta:` trailing blocks) so a parsing failure in one
//! area points at a small, independently readable/testable function instead of
//! a single ~200-line `parse`.
use syn::{
    Ident, LitStr, Token, braced, bracketed,
    parse::{Parse, ParseStream},
};

use super::field::DslField;
use super::relation::DslRelation;

pub(super) struct DslModel {
    pub name: String,
    pub table: String,
    pub pk: DslPk,
    pub enum_types: Vec<(String, String, Vec<String>)>, // (enum_name, backing_type, string_values)
    pub fields: Vec<DslField>,
    pub relations: Vec<DslRelation>,
    pub unique_together: Vec<Vec<String>>,
    pub indexes: Vec<Vec<String>>,
}

pub(super) struct DslPk {
    pub name: String,
    pub ty: String, // "i32", "i64", "uuid", "Pk"
}

/// `ModelName, table: "...", pk: name => type,` — the three always-present,
/// always-in-order header fields.
struct DslModelHeader {
    name: Ident,
    table: LitStr,
    pk_name: Ident,
    pk_ty: Ident,
}

fn parse_header(input: ParseStream) -> syn::Result<DslModelHeader> {
    let name: Ident = input.parse()?;
    input.parse::<Token![,]>()?;

    let kw: Ident = input.parse()?;
    if kw != "table" {
        return Err(syn::Error::new(kw.span(), "expected 'table'"));
    }
    input.parse::<Token![:]>()?;
    let table: LitStr = input.parse()?;
    input.parse::<Token![,]>()?;

    let kw: Ident = input.parse()?;
    if kw != "pk" {
        return Err(syn::Error::new(kw.span(), "expected 'pk'"));
    }
    input.parse::<Token![:]>()?;
    let pk_name: Ident = input.parse()?;
    input.parse::<Token![=>]>()?;
    let pk_ty: Ident = input.parse()?;
    input.parse::<Token![,]>()?;

    Ok(DslModelHeader {
        name,
        table,
        pk_name,
        pk_ty,
    })
}

/// Optional `enums: { Name [Variant, ...], ... }` block, right after the header.
/// Absent entirely if the model declares no enums.
fn parse_optional_enums_block(
    input: ParseStream,
) -> syn::Result<Vec<(String, String, Vec<String>)>> {
    let mut enum_types: Vec<(String, String, Vec<String>)> = Vec::new();
    if !input.peek(Ident) {
        return Ok(enum_types);
    }
    let peek: Ident = input.fork().parse()?;
    if peek != "enums" {
        return Ok(enum_types);
    }
    input.parse::<Ident>()?;
    input.parse::<Token![:]>()?;
    let enum_block;
    braced!(enum_block in input);
    while !enum_block.is_empty() {
        if let Ok(enum_name) = enum_block.parse::<Ident>() {
            let _ = enum_block.parse::<Token![:]>();
            // optional type: i32 | i64 (String and pg are obsolete but tolerated here)
            let backing = if enum_block.peek(Ident) {
                let ty: Ident = enum_block.fork().parse().unwrap();
                match ty.to_string().as_str() {
                    "String" | "i32" | "i64" | "pg" => {
                        enum_block.parse::<Ident>().ok();
                        ty.to_string()
                    }
                    _ => "Auto".to_string(),
                }
            } else {
                "Auto".to_string()
            };
            // [Name] or [Name="value", ...] variants
            let mut string_values: Vec<String> = Vec::new();
            if enum_block.peek(syn::token::Bracket) {
                let variants;
                bracketed!(variants in enum_block);
                while !variants.is_empty() {
                    if let Ok(vname) = variants.parse::<Ident>() {
                        if variants.peek(Token![=]) {
                            let _ = variants.parse::<Token![=]>();
                            if let Ok(lit) = variants.parse::<syn::Lit>() {
                                match lit {
                                    syn::Lit::Str(s) => string_values.push(s.value()),
                                    syn::Lit::Int(n) => string_values.push(n.to_string()),
                                    _ => string_values.push(vname.to_string()),
                                }
                            } else if variants.peek(syn::token::Paren) {
                                // tuple syntax: ("db_value", "Display") — only keep first value
                                let tuple;
                                syn::parenthesized!(tuple in variants);
                                if let Ok(syn::Lit::Str(s)) = tuple.parse::<syn::Lit>() {
                                    string_values.push(s.value());
                                } else {
                                    string_values.push(vname.to_string());
                                }
                                // consume the rest of the tuple (display label, etc.)
                                while !tuple.is_empty() {
                                    tuple.parse::<proc_macro2::TokenTree>().ok();
                                }
                            } else {
                                string_values.push(vname.to_string());
                            }
                        } else {
                            string_values.push(vname.to_string());
                        }
                    } else {
                        variants.parse::<proc_macro2::TokenTree>().ok();
                    }
                    let _ = variants.parse::<Token![,]>();
                }
            }
            enum_types.push((enum_name.to_string(), backing, string_values));
            let _ = enum_block.parse::<Token![,]>();
        } else {
            enum_block.parse::<proc_macro2::TokenTree>().ok();
        }
    }
    let _ = input.parse::<Token![,]>();
    Ok(enum_types)
}

/// The required anonymous `{ name: type [opts], ... }` fields block.
fn parse_fields_block(input: ParseStream) -> syn::Result<Vec<DslField>> {
    let mut fields = Vec::new();
    let fields_content;
    braced!(fields_content in input);
    while !fields_content.is_empty() {
        fields.push(DslField::parse(&fields_content)?);
    }
    Ok(fields)
}

/// A `[(col, col), (col, col), ...]` list, as used by both
/// `meta.unique_together` and `meta.indexes` — same shape, so parsed once here
/// instead of twice.
fn parse_column_group_list(input: ParseStream) -> syn::Result<Vec<Vec<String>>> {
    let list_content;
    bracketed!(list_content in input);
    let mut groups: Vec<Vec<String>> = Vec::new();
    while !list_content.is_empty() {
        let tuple_content;
        syn::parenthesized!(tuple_content in list_content);
        let mut group = Vec::new();
        while !tuple_content.is_empty() {
            if let Ok(f) = tuple_content.parse::<Ident>() {
                group.push(f.to_string());
            }
            let _ = tuple_content.parse::<Token![,]>();
        }
        if !group.is_empty() {
            groups.push(group);
        }
        let _ = list_content.parse::<Token![,]>();
    }
    Ok(groups)
}

/// `relations: { belongs_to: Model via col [...], has_many: Model, ... }` block
/// body — a malformed entry is skipped up to the next comma rather than
/// failing the whole model.
fn parse_relations_block(input: ParseStream) -> Vec<DslRelation> {
    let mut relations = Vec::new();
    while !input.is_empty() {
        if let Ok(rel) = DslRelation::parse(input) {
            relations.push(rel);
        } else {
            while !input.is_empty() && !input.peek(Token![,]) {
                input.parse::<proc_macro2::TokenTree>().ok();
            }
            let _ = input.parse::<Token![,]>();
        }
    }
    relations
}

/// `meta: { unique_together: [...], indexes: [...], ordering: [...], ... }`
/// block body. Only `unique_together`/`indexes` are structurally meaningful
/// here (both column-group lists, see [`parse_column_group_list`]); every
/// other key (`ordering`, `verbose_name`, `abstract`, ...) is consumed and
/// ignored — this parser only extracts what the migration generator needs.
fn parse_meta_block(input: ParseStream) -> syn::Result<(Vec<Vec<String>>, Vec<Vec<String>>)> {
    let mut unique_together: Vec<Vec<String>> = Vec::new();
    let mut indexes: Vec<Vec<String>> = Vec::new();
    while !input.is_empty() {
        let Ok(meta_key) = input.parse::<Ident>() else {
            input.parse::<proc_macro2::TokenTree>().ok();
            continue;
        };
        let _ = input.parse::<Token![:]>();
        match meta_key.to_string().as_str() {
            "unique_together" => unique_together.extend(parse_column_group_list(input)?),
            "indexes" => indexes.extend(parse_column_group_list(input)?),
            _ => {
                // ordering, verbose_name, abstract, etc. — value is one TokenTree
                input.parse::<proc_macro2::TokenTree>().ok();
            }
        }
        let _ = input.parse::<Token![,]>();
    }
    Ok((unique_together, indexes))
}

/// Trailing optional blocks after the fields block — `relations:`, `meta:`,
/// and anything else (`form_fields:`, etc.), which is present in source but
/// irrelevant to migrations and simply skipped.
fn parse_trailing_blocks(
    input: ParseStream,
) -> syn::Result<(Vec<DslRelation>, Vec<Vec<String>>, Vec<Vec<String>>)> {
    let mut relations = Vec::new();
    let mut unique_together: Vec<Vec<String>> = Vec::new();
    let mut indexes: Vec<Vec<String>> = Vec::new();

    while !input.is_empty() {
        let _ = input.parse::<Token![,]>();
        if input.is_empty() {
            break;
        }
        if !input.peek(Ident) {
            input.parse::<proc_macro2::TokenTree>().ok();
            continue;
        }
        let kw: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let block_content;
        braced!(block_content in input);

        match kw.to_string().as_str() {
            "relations" => relations.extend(parse_relations_block(&block_content)),
            "meta" => {
                let (ut, idx) = parse_meta_block(&block_content)?;
                unique_together.extend(ut);
                indexes.extend(idx);
            }
            _ => {
                // form_fields, etc. — ignored
                while !block_content.is_empty() {
                    block_content.parse::<proc_macro2::TokenTree>().ok();
                }
            }
        }
    }

    Ok((relations, unique_together, indexes))
}

impl Parse for DslModel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let header = parse_header(input)?;
        let enum_types = parse_optional_enums_block(input)?;
        let fields = parse_fields_block(input)?;
        let (relations, unique_together, indexes) = parse_trailing_blocks(input)?;

        Ok(DslModel {
            name: header.name.to_string(),
            table: header.table.value(),
            pk: DslPk {
                name: header.pk_name.to_string(),
                ty: header.pk_ty.to_string(),
            },
            enum_types,
            fields,
            relations,
            unique_together,
            indexes,
        })
    }
}
