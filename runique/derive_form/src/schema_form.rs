use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    DeriveInput, Ident, Path, Token, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

// ─── Attributs parsés depuis #[form(schema = ..., fields = [...], exclude = [...])] ───

struct FormAttrs {
    schema: Path,
    fields: Option<Vec<Ident>>,
    exclude: Option<Vec<Ident>>,
    model: Option<Path>,
}

impl Parse for FormAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema: Option<Path> = None;
        let mut fields: Option<Vec<Ident>> = None;
        let mut exclude: Option<Vec<Ident>> = None;
        let mut model: Option<Path> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "schema" => {
                    schema = Some(input.parse::<Path>()?);
                }
                "fields" => {
                    let content;
                    bracketed!(content in input);
                    let idents: Punctuated<Ident, Token![,]> =
                        content.parse_terminated(Ident::parse, Token![,])?;
                    fields = Some(idents.into_iter().collect());
                }
                "exclude" => {
                    let content;
                    bracketed!(content in input);
                    let idents: Punctuated<Ident, Token![,]> =
                        content.parse_terminated(Ident::parse, Token![,])?;
                    exclude = Some(idents.into_iter().collect());
                }
                "model" => {
                    model = Some(input.parse::<Path>()?);
                }
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("#[form]: attribut inconnu `{}`", other),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let schema = schema
            .ok_or_else(|| syn::Error::new(Span::call_site(), "#[form]: `schema` est requis"))?;

        Ok(FormAttrs {
            schema,
            fields,
            exclude,
            model,
        })
    }
}

// ─── Point d'entrée du macro ───────────────────────────────────────────────

pub(crate) fn model_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let attrs = parse_macro_input!(attr as FormAttrs);
    let schema_path = attrs.schema;

    let fields_expr = ident_list_to_expr(attrs.fields);
    let exclude_expr = ident_list_to_expr(attrs.exclude);

    let model_impls = if let Some(model_path) = attrs.model {
        quote! {
            impl ::runique::forms::FormEntity for #name {
                type Entity = #model_path;
            }
            impl #name {
                pub const objects: ::runique::macros::bdd::objects::Objects<#model_path> =
                    ::runique::macros::bdd::objects::Objects::new();
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        #[serde(transparent)]
        pub struct #name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::model_form::ModelForm for #name {
            fn schema() -> ::runique::migration::schema::ModelSchema {
                #schema_path::schema()
            }
            fn fields() -> Option<&'static [&'static str]> {
                #fields_expr
            }
            fn exclude() -> Option<&'static [&'static str]> {
                #exclude_expr
            }
        }

        #model_impls
    };

    TokenStream::from(expanded)
}

fn ident_list_to_expr(idents: Option<Vec<Ident>>) -> proc_macro2::TokenStream {
    match idents {
        Some(list) => {
            let lits: Vec<syn::LitStr> = list
                .iter()
                .map(|id| syn::LitStr::new(&id.to_string(), id.span()))
                .collect();
            quote! { Some(&[#(#lits),*]) }
        }
        None => quote! { None },
    }
}
