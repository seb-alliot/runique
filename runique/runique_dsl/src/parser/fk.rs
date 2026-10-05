//! Foreign key action keywords, read in `belongs_to: Model via column [...]`.
use crate::ast::FkAction;
use syn::{Ident, Result};

/// A foreign key action keyword — `cascade`, `set_null`, `restrict`,
/// `set_default` or `no_action`; anything else is refused.
pub(super) fn parse_fk_action(ident: &Ident) -> Result<FkAction> {
    match ident.to_string().as_str() {
        "no_action" => Ok(FkAction::NoAction),
        "cascade" => Ok(FkAction::Cascade),
        "set_null" => Ok(FkAction::SetNull),
        "restrict" => Ok(FkAction::Restrict),
        "set_default" => Ok(FkAction::SetDefault),
        other => Err(syn::Error::new(
            ident.span(),
            format!(
                "Unknown FK action: '{}'. Expected: cascade, set_null, restrict, set_default, no_action",
                other
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(src: &str) -> Result<FkAction> {
        parse_fk_action(&syn::parse_str::<Ident>(src).unwrap())
    }

    #[test]
    fn every_action_keyword() {
        assert_eq!(action("cascade").unwrap(), FkAction::Cascade);
        assert_eq!(action("set_null").unwrap(), FkAction::SetNull);
        assert_eq!(action("restrict").unwrap(), FkAction::Restrict);
        assert_eq!(action("set_default").unwrap(), FkAction::SetDefault);
        assert_eq!(action("no_action").unwrap(), FkAction::NoAction);
    }

    #[test]
    fn unknown_action_rejected() {
        assert!(action("unknown_action").is_err());
    }
}
