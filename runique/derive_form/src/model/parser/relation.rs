//! `relations: { belongs_to: ..., has_many: ..., has_one: ..., many_to_many: ... }`
//! entry parsing.
use crate::model::ast::RelationDef;
use proc_macro2;
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

impl Parse for RelationDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // belongs_to: users via user_id,
        // has_many: comments as user_comments,
        // has_one: profile as user_profile,
        // many_to_many: roles through user_roles,
        let kind: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let model: Ident = input.parse()?;

        let relation = match kind.to_string().as_str() {
            "belongs_to" => {
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(via_kw.span(), "Expected: 'via'"));
                }
                let via: Ident = input.parse()?;
                // Optional FK options: [cascade], [cascade, restrict], etc.
                // consumed here, handled by migration system
                if input.peek(syn::token::Bracket) {
                    let opts;
                    syn::bracketed!(opts in input);
                    while !opts.is_empty() {
                        opts.parse::<proc_macro2::TokenTree>().ok();
                    }
                }
                RelationDef::BelongsTo { model, via }
            }
            "has_many" => {
                // `as` is a strict Rust keyword — `input.peek(Ident)` structurally
                // never matches it, it must be peeked/consumed via `Token![as]`.
                let as_name = if input.peek(Token![as]) {
                    input.parse::<Token![as]>()?;
                    Some(input.parse::<Ident>()?)
                } else {
                    None
                };
                RelationDef::HasMany { model, as_name }
            }
            "has_one" => {
                let as_name = if input.peek(Token![as]) {
                    input.parse::<Token![as]>()?;
                    Some(input.parse::<Ident>()?)
                } else {
                    None
                };
                RelationDef::HasOne { model, as_name }
            }
            "many_to_many" => {
                let through_kw: Ident = input.parse()?;
                if through_kw != "through" {
                    return Err(syn::Error::new(through_kw.span(), "Expected: 'through'"));
                }
                let through: Ident = input.parse()?;

                // via ViaIdent — kept in the DSL grammar for readability at the
                // call site (`many_to_many: Target through Junction via col`),
                // but the column name itself is no longer stored: the generator
                // derives the through entity's self-pointing relation from the
                // declaring model's own name instead, which is exact rather
                // than guessed from this column.
                let via_kw: Ident = input.parse()?;
                if via_kw != "via" {
                    return Err(syn::Error::new(via_kw.span(), "Expected: 'via'"));
                }
                let _via_self: Ident = input.parse()?;

                RelationDef::ManyToMany { model, through }
            }
            other => {
                return Err(syn::Error::new(
                    kind.span(),
                    format!(
                        "Unknown relation: '{}'. Expected: belongs_to, has_many, has_one, many_to_many",
                        other
                    ),
                ));
            }
        };

        let _ = input.parse::<Token![,]>();
        Ok(relation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<RelationDef> {
        syn::parse_str::<RelationDef>(src)
    }

    #[test]
    fn belongs_to_basic() {
        let rel = parse("belongs_to: User via user_id").unwrap();
        match rel {
            RelationDef::BelongsTo { model, via } => {
                assert_eq!(model.to_string(), "User");
                assert_eq!(via.to_string(), "user_id");
            }
            _ => panic!("expected BelongsTo"),
        }
    }

    #[test]
    fn belongs_to_missing_via_rejected() {
        assert!(parse("belongs_to: User").is_err());
    }

    #[test]
    fn belongs_to_with_fk_options_consumed() {
        // Options are consumed here but interpreted by the migration system,
        // not stored on RelationDef — just confirm they don't break parsing.
        assert!(parse("belongs_to: User via user_id [cascade]").is_ok());
    }

    #[test]
    fn has_many_without_alias() {
        let rel = parse("has_many: Comment").unwrap();
        match rel {
            RelationDef::HasMany { model, as_name } => {
                assert_eq!(model.to_string(), "Comment");
                assert!(as_name.is_none());
            }
            _ => panic!("expected HasMany"),
        }
    }

    #[test]
    fn has_many_with_as_alias() {
        let rel = parse("has_many: Comment as user_comments").unwrap();
        match rel {
            RelationDef::HasMany { as_name, .. } => {
                assert_eq!(as_name.unwrap().to_string(), "user_comments");
            }
            _ => panic!("expected HasMany"),
        }
    }

    #[test]
    fn has_one_without_alias() {
        let rel = parse("has_one: Profile").unwrap();
        assert!(matches!(rel, RelationDef::HasOne { as_name: None, .. }));
    }

    #[test]
    fn has_one_with_as_alias() {
        let rel = parse("has_one: Profile as user_profile").unwrap();
        match rel {
            RelationDef::HasOne { as_name, .. } => {
                assert_eq!(as_name.unwrap().to_string(), "user_profile");
            }
            _ => panic!("expected HasOne"),
        }
    }

    #[test]
    fn many_to_many_basic() {
        let rel = parse("many_to_many: Role through user_roles via user_id").unwrap();
        match rel {
            RelationDef::ManyToMany { model, through } => {
                assert_eq!(model.to_string(), "Role");
                assert_eq!(through.to_string(), "user_roles");
            }
            _ => panic!("expected ManyToMany"),
        }
    }

    #[test]
    fn many_to_many_via_column_required_but_not_stored() {
        // `via <col>` stays mandatory in the grammar (readability at the call
        // site), even though its value is no longer kept on `RelationDef`.
        assert!(parse("many_to_many: Role through user_roles via user_id").is_ok());
        assert!(parse("many_to_many: Role through user_roles via").is_err());
    }

    #[test]
    fn many_to_many_missing_through_rejected() {
        assert!(parse("many_to_many: Role via user_id").is_err());
    }

    #[test]
    fn many_to_many_missing_via_rejected() {
        assert!(parse("many_to_many: Role through user_roles").is_err());
    }

    #[test]
    fn unknown_relation_kind_rejected() {
        assert!(parse("unknown_kind: Something").is_err());
    }
}
