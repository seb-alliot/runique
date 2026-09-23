//! `meta: { ordering:, unique_together:, indexes:, verbose_name:, ... }`
//! block parsing.
use crate::model::ast::MetaDef;
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
        let mut abstract_model = false;
        let mut indexes = Vec::new();

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
                // `abstract` is a reserved Rust keyword — `Ident::parse` structurally
                // cannot accept it, so this DSL key was unreachable under that name
                // (found via a test exercising it directly). `is_abstract` isn't reserved.
                "is_abstract" => {
                    let b: syn::LitBool = input.parse()?;
                    abstract_model = b.value();
                }
                "indexes" => indexes = parse_ident_group_list(input)?,
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
            abstract_model,
            indexes,
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

    #[test]
    fn abstract_flag() {
        let m = parse("is_abstract: true").unwrap();
        assert!(m.abstract_model);
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
}
