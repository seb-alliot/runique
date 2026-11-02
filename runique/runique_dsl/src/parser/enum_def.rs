//! `enums: { Name [Variant, ...], ... }` entry parsing — one `EnumDef` per
//! `Name: [...]` line.
use crate::ast::{EnumBackingType, EnumDef, EnumVariant};
use std::collections::{HashMap, HashSet};
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

/// A string enum reads a submitted value back by its stored value, its name or
/// its label (`FromStr`, first match wins): no two variants may share one, or
/// a value the form offered for one variant is read back as another.
fn validate_string_aliases(
    name: &Ident,
    backing_type: &EnumBackingType,
    variants: &[EnumVariant],
) -> Result<()> {
    if !matches!(backing_type, EnumBackingType::Auto) {
        return Ok(());
    }
    let mut owners: HashMap<String, &Ident> = HashMap::new();
    for variant in variants {
        let v = &variant.name;
        let aliases: HashSet<String> = [variant.db_str(), v.to_string(), variant.display_str()]
            .into_iter()
            .collect();
        for alias in aliases {
            if let Some(other) = owners.insert(alias.clone(), v) {
                return Err(syn::Error::new(
                    v.span(),
                    format!(
                        "Enum '{name}': \"{alias}\" is a value, name or label of both '{other}' and '{v}' — a submitted \"{alias}\" would be read as '{other}'"
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// An `i32`/`i64` enum stores each variant as its number: every variant needs
/// one, within the type's range, and no two may share it — otherwise rows
/// written as one variant would be read back as another.
fn validate_int_values(
    name: &Ident,
    backing_type: &EnumBackingType,
    variants: &[EnumVariant],
) -> Result<()> {
    let (ty, min, max) = match backing_type {
        EnumBackingType::I8 => ("i8", i64::from(i8::MIN), i64::from(i8::MAX)),
        EnumBackingType::I16 => ("i16", i64::from(i16::MIN), i64::from(i16::MAX)),
        EnumBackingType::I32 => ("i32", i64::from(i32::MIN), i64::from(i32::MAX)),
        EnumBackingType::I64 => ("i64", i64::MIN, i64::MAX),
        EnumBackingType::Auto => return Ok(()),
    };
    let mut seen: HashSet<i64> = HashSet::new();
    for variant in variants {
        let v = &variant.name;
        let Some(value) = variant.int_value().filter(|n| (min..=max).contains(n)) else {
            return Err(syn::Error::new(
                v.span(),
                format!(
                    "Enum '{name}' is `{ty}`: variant '{v}' needs an integer value that fits in `{ty}` — `{v} = 1` or `{v} = (1, \"label\")`"
                ),
            ));
        };
        if !seen.insert(value) {
            return Err(syn::Error::new(
                v.span(),
                format!("Enum '{name}': variant '{v}' reuses the value {value}"),
            ));
        }
    }
    Ok(())
}

impl Parse for EnumDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // Status: [Active, Inactive] or Status: String [Fix="fix"] or Priority: i32 [Low=1]
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        // Optional type: i8 | i16 | i32 | i64 (otherwise Auto — detected from .env)
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
                "i8" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I8
                }
                "i16" => {
                    input.parse::<Ident>()?;
                    EnumBackingType::I16
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
        validate_int_values(&name, &backing_type, &variants)?;
        validate_string_aliases(&name, &backing_type, &variants)?;
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
        let e = parse("Priority: i32 [Low = 1, High = (2, \"Haute\")]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I32));
        let values: Vec<_> = e.variants.iter().map(|v| v.int_value()).collect();
        assert_eq!(values, [Some(1), Some(2)]);
    }

    #[test]
    fn explicit_i64_backing_type() {
        let e = parse("BigEnum: i64 [A = 1, B = 5000000000]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I64));
    }

    #[test]
    fn integer_enum_needs_a_value_per_variant() {
        assert!(parse("Priority: i32 [Low, High]").is_err());
        assert!(parse("Priority: i32 [Low = 1, High]").is_err());
        assert!(parse("Priority: i32 [Low: \"Basse\"]").is_err());
        assert!(parse("Priority: i32 [Low = \"low\"]").is_err());
    }

    #[test]
    fn small_integer_backing_types() {
        let e = parse("Level: i8 [Low = 1, High = 127]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I8));
        let e = parse("Code: i16 [A = -300, B = 300]").unwrap();
        assert!(matches!(e.backing_type, EnumBackingType::I16));
        assert!(parse("Level: i8 [Low = 128]").is_err(), "out of i8 range");
        assert!(parse("Code: i16 [A = 40000]").is_err(), "out of i16 range");
    }

    #[test]
    fn integer_enum_value_must_fit_and_be_unique() {
        assert!(parse("Priority: i32 [Low = 3000000000]").is_err());
        assert!(parse("Priority: i32 [Low = 1, High = 1]").is_err());
    }

    #[test]
    fn deprecated_string_backing_type_rejected() {
        let Err(e) = parse("Status: String [Active]") else {
            panic!("String backing type is refused");
        };
        assert!(e.to_string().contains("deprecated"), "says why: {e}");
    }

    #[test]
    fn duplicate_variant_rejected() {
        assert!(parse("Status: [Active, Active]").is_err());
    }

    #[test]
    fn label_matching_another_variants_value_rejected() {
        let Err(e) = parse(r#"Status: [Admin: "draft", Draft="draft"]"#) else {
            panic!("\"draft\" would read back as Admin");
        };
        assert!(
            e.to_string().contains("'Admin' and 'Draft'"),
            "names both: {e}"
        );
    }

    #[test]
    fn value_matching_another_variants_name_rejected() {
        assert!(parse(r#"Status: [Draft, Published="Draft"]"#).is_err());
    }

    #[test]
    fn shared_label_rejected() {
        assert!(parse(r#"Status: [Draft: "Pending", Review: "Pending"]"#).is_err());
    }

    #[test]
    fn a_variant_may_repeat_its_own_aliases() {
        assert!(parse(r#"Status: [Draft="Draft", Published=("published", "Published")]"#).is_ok());
    }

    #[test]
    fn int_enums_are_not_checked_for_string_aliases() {
        // Their `FromStr` matches the variant name only.
        assert!(parse(r#"Level: i32 [Low=(1, "Same"), High=(2, "Same")]"#).is_ok());
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
