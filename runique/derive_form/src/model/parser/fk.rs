//! `fk(table.column, action)` foreign key declaration parsing.
use crate::model::ast::{FkAction, FkDef};
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

impl Parse for FkDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // users.id, cascade
        let table: Ident = input.parse()?;
        input.parse::<Token![.]>()?;
        let column: Ident = input.parse()?;
        input.parse::<Token![,]>()?;
        let action_ident: Ident = input.parse()?;
        let action = match action_ident.to_string().as_str() {
            "cascade" => FkAction::Cascade,
            "set_null" => FkAction::SetNull,
            "restrict" => FkAction::Restrict,
            "set_default" => FkAction::SetDefault,
            other => {
                return Err(syn::Error::new(
                    action_ident.span(),
                    format!(
                        "Unknown FK action: '{}'. Expected: cascade, set_null, restrict, set_default",
                        other
                    ),
                ));
            }
        };
        Ok(FkDef {
            table,
            column,
            action,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<FkDef> {
        syn::parse_str::<FkDef>(src)
    }

    #[test]
    fn cascade_action() {
        let fk = parse("users.id, cascade").unwrap();
        assert_eq!(fk.table.to_string(), "users");
        assert_eq!(fk.column.to_string(), "id");
        assert!(matches!(fk.action, FkAction::Cascade));
    }

    #[test]
    fn set_null_action() {
        let fk = parse("users.id, set_null").unwrap();
        assert!(matches!(fk.action, FkAction::SetNull));
    }

    #[test]
    fn restrict_action() {
        let fk = parse("users.id, restrict").unwrap();
        assert!(matches!(fk.action, FkAction::Restrict));
    }

    #[test]
    fn set_default_action() {
        let fk = parse("users.id, set_default").unwrap();
        assert!(matches!(fk.action, FkAction::SetDefault));
    }

    #[test]
    fn unknown_action_rejected() {
        assert!(parse("users.id, unknown_action").is_err());
    }

    #[test]
    fn missing_dot_rejected() {
        assert!(parse("users_id, cascade").is_err());
    }
}
