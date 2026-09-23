//! `pk: name => type` primary key declaration parsing.
use crate::model::ast::{PkDef, PkType};
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

impl Parse for PkDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // id => i32
        let name: Ident = input.parse()?;
        input.parse::<Token![=>]>()?;
        let ty_ident: Ident = input.parse()?;
        let ty = match ty_ident.to_string().as_str() {
            "i32" => PkType::I32,
            "i64" => PkType::I64,
            "uuid" => PkType::Uuid,
            "Pk" => {
                #[cfg(feature = "pk-uuid")]
                {
                    PkType::Uuid
                }
                #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
                {
                    PkType::I64
                }
                #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
                {
                    PkType::I32
                }
            }
            other => {
                return Err(syn::Error::new(
                    ty_ident.span(),
                    format!("Unknown PK type: '{}'. Expected: i32, i64, Pk, uuid", other),
                ));
            }
        };
        Ok(PkDef { name, ty })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(src: &str) -> Result<PkDef> {
        syn::parse_str::<PkDef>(src)
    }

    #[test]
    fn i32_type() {
        let pk = parse("id => i32").unwrap();
        assert_eq!(pk.name.to_string(), "id");
        assert!(matches!(pk.ty, PkType::I32));
    }

    #[test]
    fn i64_type() {
        let pk = parse("id => i64").unwrap();
        assert!(matches!(pk.ty, PkType::I64));
    }

    #[test]
    fn uuid_type() {
        let pk = parse("id => uuid").unwrap();
        assert!(matches!(pk.ty, PkType::Uuid));
    }

    #[test]
    fn pk_alias_resolves_to_a_concrete_type() {
        // Resolves to whichever of i32/i64/uuid is active for the compiled
        // feature set — just confirm it parses to *some* concrete PkType,
        // never left unresolved.
        let pk = parse("id => Pk").unwrap();
        assert!(matches!(pk.ty, PkType::I32 | PkType::I64 | PkType::Uuid));
    }

    #[test]
    fn unknown_type_rejected() {
        assert!(parse("id => not_a_type").is_err());
    }

    #[test]
    fn missing_arrow_rejected() {
        assert!(parse("id i32").is_err());
    }
}
