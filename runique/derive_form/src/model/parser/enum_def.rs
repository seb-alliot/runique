//! `enums: { Name [Variant, ...], ... }` entry parsing — one `EnumDef` per
//! `Name: [...]` line.
use crate::model::ast::{EnumBackingType, EnumDef, EnumVariant};
use std::collections::HashSet;
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

impl Parse for EnumDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // Status: [Active, Inactive] or Status: String [Fix="fix"] or Priority: i32 [Low=1]
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Optional type: String | i32 | i64 (otherwise Auto — detected from .env)
        let backing_type = if input.peek(Ident) {
            let ty: Ident = input.fork().parse()?;
            match ty.to_string().as_str() {
                "String" => {
                    let ident: Ident = input.parse()?;
                    return Err(syn::Error::new(
                        ident.span(),
                        "`String` is deprecated as an enum type. \
                        Remove it — the correct behavior is automatically detected \
                        from DATABASE_URL in `.env` (native Postgres or VARCHAR).",
                    ));
                }
                "i32" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I32
                }
                "i64" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I64
                }
                _ => EnumBackingType::Auto,
            }
        } else {
            EnumBackingType::Auto
        };

        let content;
        syn::bracketed!(content in input);
        let mut variants = Vec::new();
        let mut seen_variant_names: HashSet<String> = HashSet::new();
        while !content.is_empty() {
            let variant_name: Ident = content.parse()?;
            if !seen_variant_names.insert(variant_name.to_string()) {
                return Err(syn::Error::new(
                    variant_name.span(),
                    format!(
                        "Variant '{}' is declared more than once in enum '{}'",
                        variant_name, name
                    ),
                ));
            }
            let (value, label) = if content.peek(Token![:]) {
                content.parse::<Token![:]>()?;
                (None, Some(content.parse::<syn::Lit>()?))
            } else if content.peek(Token![=]) {
                content.parse::<Token![=]>()?;
                if content.peek(syn::token::Paren) {
                    let inner;
                    syn::parenthesized!(inner in content);
                    let db_val: syn::Lit = inner.parse()?;
                    inner.parse::<Token![,]>()?;
                    let lbl: syn::Lit = inner.parse()?;
                    (Some(db_val), Some(lbl))
                } else {
                    (Some(content.parse::<syn::Lit>()?), None)
                }
            } else {
                (None, None)
            };
            variants.push(EnumVariant {
                name: variant_name,
                value,
                label,
            });
            let _ = content.parse::<Token![,]>();
        }
        if variants.is_empty() {
            return Err(syn::Error::new(
                name.span(),
                format!("Enum '{}' must declare at least one variant", name),
            ));
        }
        let _ = input.parse::<Token![,]>();
        Ok(EnumDef {
            name,
            backing_type,
            variants,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<EnumDef> {
        syn::parse_str::<EnumDef>(src)
    }

    #[test]
    fn simple_variants_no_backing_type() {
        let e = parse("Status: [Active, Inactive]").unwrap();
        assert_eq!(e.name.to_string(), "Status");
        assert!(matches!(e.backing_type, EnumBackingType::Auto));
        assert_eq!(e.variants.len(), 2);
    }

    #[test]
    fn explicit_i32_backing_type() {
        let e = parse("Priority: i32 [Low, High]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I32));
    }

    #[test]
    fn explicit_i64_backing_type() {
        let e = parse("BigEnum: i64 [A, B]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I64));
    }

    #[test]
    fn deprecated_string_backing_type_rejected() {
        assert!(parse("Status: String [Active]").is_err());
    }

    #[test]
    fn duplicate_variant_rejected() {
        assert!(parse("Status: [Active, Active]").is_err());
    }

    #[test]
    fn empty_variants_rejected() {
        assert!(parse("Status: []").is_err());
    }

    #[test]
    fn variant_with_explicit_int_value_has_no_separate_label() {
        let e = parse("Status: [Active=1, Inactive=0]").unwrap();
        assert!(e.variants[0].label.is_none());
        assert!(matches!(&e.variants[0].value, Some(syn::Lit::Int(_))));
    }

    #[test]
    fn variant_with_tuple_value_and_label() {
        let e = parse(r#"Status: [Active=("active", "Actif")]"#).unwrap();
        let v = &e.variants[0];
        assert!(matches!(&v.value, Some(syn::Lit::Str(s)) if s.value() == "active"));
        assert!(matches!(&v.label, Some(syn::Lit::Str(s)) if s.value() == "Actif"));
    }

    #[test]
    fn variant_with_colon_label_only() {
        let e = parse(r#"Status: [Active: "Actif"]"#).unwrap();
        let v = &e.variants[0];
        assert!(v.value.is_none());
        assert!(matches!(&v.label, Some(syn::Lit::Str(s)) if s.value() == "Actif"));
    }
}
