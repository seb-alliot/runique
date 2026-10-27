//! `meta: { ordering:, unique_together:, indexes:, verbose_name:, ... }`
//! block parsing.
use crate::ast::MetaDef;
use syn::{
    Ident, LitStr, Result, Token,
    parse::{Parse, ParseStream},
};

/// A `[(col, col), (col, col), ...]` list, as used by both `unique_together`
/// and `indexes` — same shape, parsed once here instead of twice.
fn parse_ident_group_list(input: ParseStream) -> Result<Vec<Vec<Ident>>> {
    let content;
    syn::bracketed!(content in input);
    let mut groups = Vec::new();
    while !content.is_empty() {
        let tuple_content;
        syn::parenthesized!(tuple_content in content);
        let mut group = Vec::new();
        while !tuple_content.is_empty() {
            group.push(tuple_content.parse::<Ident>()?);
            let _ = tuple_content.parse::<Token![,]>();
        }
        groups.push(group);
        let _ = content.parse::<Token![,]>();
    }
    Ok(groups)
}

impl Parse for MetaDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut ordering = Vec::new();
        let mut unique_together = Vec::new();
        let mut verbose_name = None;
        let mut verbose_name_plural = None;
        let mut indexes = Vec::new();
        let mut owner = None;
        let mut label = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "ordering" => {
                    let content;
                    syn::bracketed!(content in input);
                    while !content.is_empty() {
                        // - = DESC
                        let desc = content.peek(Token![-]);
                        if desc {
                            content.parse::<Token![-]>()?;
                        }
                        let field: Ident = content.parse()?;
                        ordering.push((desc, field));
                        let _ = content.parse::<Token![,]>();
                    }
                }
                "unique_together" => unique_together = parse_ident_group_list(input)?,
                "verbose_name" => {
                    let s: LitStr = input.parse()?;
                    verbose_name = Some(s.value());
                }
                "verbose_name_plural" => {
                    let s: LitStr = input.parse()?;
                    verbose_name_plural = Some(s.value());
                }
                "indexes" => indexes = parse_ident_group_list(input)?,
                "owner" => owner = Some(input.parse::<Ident>()?),
                "label" => label = Some(input.parse::<Ident>()?),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown meta key: '{}'", other),
                    ));
                }
            }

            let _ = input.parse::<Token![,]>();
        }

        Ok(MetaDef {
            ordering,
            unique_together,
            verbose_name,
            verbose_name_plural,
            indexes,
            owner,
            label,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<MetaDef> {
        syn::parse_str::<MetaDef>(src)
    }

    #[test]
    fn ordering_ascending_and_descending() {
        let m = parse("ordering: [name, -created_at]").unwrap();
        assert_eq!(m.ordering.len(), 2);
        assert!(!m.ordering[0].0, "name should be ascending");
        assert_eq!(m.ordering[0].1.to_string(), "name");
        assert!(m.ordering[1].0, "-created_at should be descending");
        assert_eq!(m.ordering[1].1.to_string(), "created_at");
    }

    #[test]
    fn unique_together_tuple_list() {
        let m = parse("unique_together: [(a, b), (c, d)]").unwrap();
        assert_eq!(m.unique_together.len(), 2);
        assert_eq!(m.unique_together[0].len(), 2);
    }

    #[test]
    fn indexes_tuple_list() {
        let m = parse("indexes: [(a, b)]").unwrap();
        assert_eq!(m.indexes.len(), 1);
    }

    #[test]
    fn verbose_name() {
        let m = parse(r#"verbose_name: "Article""#).unwrap();
        assert_eq!(m.verbose_name.as_deref(), Some("Article"));
    }

    #[test]
    fn verbose_name_plural() {
        let m = parse(r#"verbose_name_plural: "Articles""#).unwrap();
        assert_eq!(m.verbose_name_plural.as_deref(), Some("Articles"));
    }

    // Abstract models (Django's `abstract = True`) don't exist in Runique:
    // there's no model inheritance for them to feed. The key used to be read
    // and then ignored, so the table was generated anyway.
    #[test]
    fn is_abstract_rejected() {
        assert!(parse("is_abstract: true").is_err());
    }

    #[test]
    fn combined_keys() {
        let m = parse(r#"ordering: [-created_at], verbose_name: "Article","#).unwrap();
        assert_eq!(m.ordering.len(), 1);
        assert_eq!(m.verbose_name.as_deref(), Some("Article"));
    }

    #[test]
    fn unknown_key_rejected() {
        assert!(parse("not_a_key: [1]").is_err());
    }

    #[test]
    fn label_is_read_as_a_field_name() {
        let m = parse("label: titre").unwrap();
        assert_eq!(m.label.map(|l| l.to_string()).as_deref(), Some("titre"));
    }

    #[test]
    fn label_absent_is_none() {
        assert!(parse("ordering: [name]").unwrap().label.is_none());
        assert!(parse("").unwrap().label.is_none());
    }

    #[test]
    fn label_combined_with_owner_and_ordering() {
        let m = parse("owner: user_id, label: titre, ordering: [-id]").unwrap();
        assert_eq!(m.owner.map(|o| o.to_string()).as_deref(), Some("user_id"));
        assert_eq!(m.label.map(|l| l.to_string()).as_deref(), Some("titre"));
        assert_eq!(m.ordering.len(), 1);
    }

    #[test]
    fn label_last_without_trailing_comma_and_with_one() {
        assert!(
            parse("ordering: [id], label: titre")
                .unwrap()
                .label
                .is_some()
        );
        assert!(parse("label: titre,").unwrap().label.is_some());
    }

    #[test]
    fn label_does_not_set_owner_nor_the_other_way_round() {
        let m = parse("label: titre").unwrap();
        assert!(m.owner.is_none());
        let m = parse("owner: user_id").unwrap();
        assert!(m.label.is_none());
    }

    #[test]
    fn label_wrong_forms_rejected() {
        assert!(parse(r#"label: "titre""#).is_err(), "a string");
        assert!(parse("label: [titre]").is_err(), "a list");
        assert!(parse("label: 1").is_err(), "a number");
        assert!(parse("label:").is_err(), "no value");
        assert!(parse("label titre").is_err(), "no colon");
    }
}
