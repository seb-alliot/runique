//! `extend!{}`: adds fields to a framework table. Reuses the `model!{}` field
//! and enum grammar.
use crate::ast::{EnumDef, FormFieldDecl};
use syn::{Ident, LitStr, Token, braced, parse::Parse, parse::ParseStream};

/// A parsed `extend!{}`: the framework table it extends, and the enums and
/// fields it adds.
pub struct ExtendDsl {
    pub table: String,
    pub enums: Vec<EnumDef>,
    pub fields: Vec<FormFieldDecl>,
}

impl Parse for ExtendDsl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let kw: Ident = input.parse()?;
        if kw != "table" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'table'"));
        }
        input.parse::<Token![:]>()?;
        let table: LitStr = input.parse()?;
        input.parse::<Token![,]>()?;

        // enums: { ... } optional — reuses the model parser (EnumDef: Parse).
        let mut enums = Vec::new();
        if input.peek(Ident) {
            let peek: Ident = input.fork().parse()?;
            if peek == "enums" {
                input.parse::<Ident>()?;
                input.parse::<Token![:]>()?;
                let enum_content;
                braced!(enum_content in input);
                while !enum_content.is_empty() {
                    enums.push(EnumDef::parse(&enum_content)?);
                }
                let _ = input.parse::<Token![,]>();
            }
        }

        let kw: Ident = input.parse()?;
        if kw != "fields" {
            return Err(syn::Error::new(kw.span(), "extend!{}: expected 'fields'"));
        }
        input.parse::<Token![:]>()?;
        let fields_content;
        braced!(fields_content in input);

        let mut fields = Vec::new();
        while !fields_content.is_empty() {
            fields.push(FormFieldDecl::parse(&fields_content)?);
        }
        let _ = input.parse::<Token![,]>();

        Ok(ExtendDsl {
            table: table.value(),
            enums,
            fields,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ExtendDsl;

    fn parse(src: &str) -> syn::Result<ExtendDsl> {
        syn::parse_str::<ExtendDsl>(src)
    }

    #[test]
    fn table_enums_and_fields_are_read() {
        let dsl = parse(
            r#"table: "eihwaz_users", enums: { A: [X], B: [Y, Z] }, fields: { bio: textarea, age: int, }"#,
        )
        .expect("parses");
        assert_eq!(dsl.table, "eihwaz_users");
        assert_eq!(dsl.enums.len(), 2);
        assert_eq!(dsl.fields.len(), 2);
    }

    #[test]
    fn enums_block_is_optional() {
        let dsl = parse(r#"table: "eihwaz_users", fields: { bio: textarea }"#).expect("parses");
        assert!(dsl.enums.is_empty());
        assert_eq!(dsl.fields.len(), 1);
    }

    #[test]
    fn misplaced_keywords_are_refused() {
        assert!(parse(r#"tbl: "eihwaz_users", fields: { bio: text }"#).is_err());
        assert!(parse(r#"table: "eihwaz_users", columns: { bio: text }"#).is_err());
    }
}
