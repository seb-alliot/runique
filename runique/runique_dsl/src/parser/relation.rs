//! `relations: { belongs_to: ..., has_many: ..., has_one: ..., many_to_many: ... }`
//! entry parsing.
use super::fk::parse_fk_action;
use crate::ast::{FkAction, RelationDef};
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
                // [on_delete] or [on_delete, on_update]
                let mut actions = Vec::new();
                if input.peek(syn::token::Bracket) {
                    let opts;
                    syn::bracketed!(opts in input);
                    let parsed = opts.parse_terminated(Ident::parse, Token![,])?;
                    if parsed.len() > 2 {
                        return Err(syn::Error::new(
                            via.span(),
                            "belongs_to takes at most two actions: [on_delete, on_update]",
                        ));
                    }
                    for ident in &parsed {
                        actions.push(parse_fk_action(ident)?);
                    }
                }
                let mut actions = actions.into_iter();
                RelationDef::BelongsTo {
                    model,
                    via,
                    on_delete: actions.next().unwrap_or(FkAction::NoAction),
                    on_update: actions.next().unwrap_or(FkAction::NoAction),
                }
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
            RelationDef::BelongsTo { model, via, .. } => {
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
    fn belongs_to_actions_are_read() {
        let RelationDef::BelongsTo {
            on_delete,
            on_update,
            ..
        } = parse("belongs_to: User via user_id [cascade, set_null]").unwrap()
        else {
            panic!("expected BelongsTo");
        };
        assert_eq!(on_delete, FkAction::Cascade);
        assert_eq!(on_update, FkAction::SetNull);
    }

    #[test]
    fn belongs_to_actions_default_to_no_action() {
        let RelationDef::BelongsTo {
            on_delete,
            on_update,
            ..
        } = parse("belongs_to: User via user_id").unwrap()
        else {
            panic!("expected BelongsTo");
        };
        assert_eq!(on_delete, FkAction::NoAction);
        assert_eq!(on_update, FkAction::NoAction);
    }

    #[test]
    fn belongs_to_unknown_or_extra_action_rejected() {
        assert!(parse("belongs_to: User via user_id [cascad]").is_err());
        assert!(parse("belongs_to: User via user_id [cascade, cascade, cascade]").is_err());
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
