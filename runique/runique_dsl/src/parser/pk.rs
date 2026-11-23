//! `pk: name => type` primary key declaration parsing.
use crate::ast::{PkDef, PkType};
use std::cell::Cell;
use syn::{
    Ident, Result, Token,
    parse::{Parse, ParseStream},
};

thread_local! {
    static PROJECT_PK: Cell<Option<PkType>> = const { Cell::new(None) };
}

/// What `Pk` stands for: the type given by [`with_project_pk`] while it runs,
/// else the one of this build's features (`pk-uuid`, `big-pk`, or `i32`).
pub(crate) fn pk_alias() -> PkType {
    if let Some(ty) = PROJECT_PK.with(Cell::get) {
        return ty;
    }
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

/// Reads models with `Pk` standing for `ty`, for as long as `read` runs.
///
/// For a reader that isn't compiled with the project it reads: the `runique`
/// command serves every project with one build, so its own features say
/// nothing of the project's. A macro never calls this: it is compiled with
/// the project, and its features are the project's.
///
/// Only the thread that calls it sees `ty`, and the former value is back when
/// `read` returns, or panics.
pub fn with_project_pk<T>(ty: PkType, read: impl FnOnce() -> T) -> T {
    struct Restore(Option<PkType>);
    impl Drop for Restore {
        fn drop(&mut self) {
            PROJECT_PK.with(|pk| pk.set(self.0));
        }
    }
    let _restore = Restore(PROJECT_PK.with(|pk| pk.replace(Some(ty))));
    read()
}

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
            "Pk" => pk_alias(),
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

    // `Pk` follows the project's type while `with_project_pk` runs, whatever
    // this build's features, and the build's own is back afterwards.
    #[test]
    fn pk_alias_follows_the_project_while_it_is_read() {
        let built = parse("id => Pk").unwrap().ty;
        for ty in [PkType::I32, PkType::I64, PkType::Uuid] {
            let read = with_project_pk(ty, || parse("id => Pk").unwrap().ty);
            assert_eq!(read, ty);
            assert_eq!(parse("id => Pk").unwrap().ty, built, "back after {ty:?}");
        }
    }

    // A type written out is never touched.
    #[test]
    fn a_written_type_ignores_the_project() {
        with_project_pk(PkType::Uuid, || {
            assert_eq!(parse("id => i32").unwrap().ty, PkType::I32);
            assert_eq!(parse("id => i64").unwrap().ty, PkType::I64);
        });
        with_project_pk(PkType::I32, || {
            assert_eq!(parse("id => uuid").unwrap().ty, PkType::Uuid);
        });
    }

    // One inside another: the inner one wins, then the outer one is back.
    #[test]
    fn nested_project_types_unwind_in_order() {
        with_project_pk(PkType::I64, || {
            with_project_pk(PkType::Uuid, || {
                assert_eq!(parse("id => Pk").unwrap().ty, PkType::Uuid);
            });
            assert_eq!(parse("id => Pk").unwrap().ty, PkType::I64);
        });
    }

    // A panic while reading doesn't leave the project's type behind.
    #[test]
    fn a_panic_while_reading_restores_the_type() {
        let built = parse("id => Pk").unwrap().ty;
        let other = if built == PkType::Uuid { PkType::I32 } else { PkType::Uuid };
        let outcome = std::panic::catch_unwind(|| with_project_pk(other, || -> () { panic!("reading") }));
        assert!(outcome.is_err());
        assert_eq!(parse("id => Pk").unwrap().ty, built);
    }

    // Another thread keeps this build's type.
    #[test]
    fn the_project_type_stays_in_its_thread() {
        let built = parse("id => Pk").unwrap().ty;
        let other = if built == PkType::Uuid { PkType::I32 } else { PkType::Uuid };
        let elsewhere = with_project_pk(other, || {
            std::thread::spawn(|| parse("id => Pk").unwrap().ty)
                .join()
                .unwrap()
        });
        assert_eq!(elsewhere, built);
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
