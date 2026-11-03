use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{
    DeriveInput, Ident, Path, Token, bracketed,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

// ─── Attributes parsed from #[form(schema = ..., fields = [...] | fields = All | exclude = [...])] ───

/// Which columns the form takes: listed, all of them, or all but some. One of
/// the three is required — a form taking every column unannounced would take
/// any column added to the model later.
enum Columns {
    Only(Vec<Ident>),
    All,
    Except(Vec<Ident>),
}

struct FormAttrs {
    schema: Path,
    columns: Columns,
}

fn ident_list(input: ParseStream) -> syn::Result<Vec<Ident>> {
    let content;
    bracketed!(content in input);
    let idents: Punctuated<Ident, Token![,]> = content.parse_terminated(Ident::parse, Token![,])?;
    Ok(idents.into_iter().collect())
}

impl Parse for FormAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut schema: Option<Path> = None;
        let mut columns: Option<Columns> = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            let declared = match key.to_string().as_str() {
                "schema" => {
                    schema = Some(input.parse::<Path>()?);
                    None
                }
                "fields" if input.peek(Ident) => {
                    let all: Ident = input.parse()?;
                    if all != "All" {
                        return Err(syn::Error::new(
                            all.span(),
                            "#[form]: `fields` takes a list `[a, b]` or `All`",
                        ));
                    }
                    Some(Columns::All)
                }
                "fields" => Some(Columns::Only(ident_list(input)?)),
                "exclude" => Some(Columns::Except(ident_list(input)?)),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("#[form]: unknown attribute `{}`", other),
                    ));
                }
            };
            if let Some(declared) = declared
                && columns.replace(declared).is_some()
            {
                return Err(syn::Error::new(
                    key.span(),
                    "#[form]: `fields` and `exclude` can't be used together — list the fields, \
                     or exclude some from all of them",
                ));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        let schema = schema
            .ok_or_else(|| syn::Error::new(Span::call_site(), "#[form]: `schema` is required"))?;
        let columns = columns.ok_or_else(|| {
            syn::Error::new(
                Span::call_site(),
                "#[form]: incomplete declaration — say which fields the form takes: \
                 `fields = [a, b]`, `exclude = [a, b]` or `fields = All`",
            )
        })?;

        Ok(FormAttrs { schema, columns })
    }
}

// ─── Macro entry point ───────────────────────────────────────────────

pub(crate) fn model_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let attrs = parse_macro_input!(attr as FormAttrs);
    let schema_path = attrs.schema;

    let (fields, exclude) = match attrs.columns {
        Columns::Only(list) => (Some(list), None),
        Columns::All => (None, None),
        Columns::Except(list) => (None, Some(list)),
    };
    // Each name a column of the entity, checked by the compiler: the error
    // lands on the name, not on a panic the first time the form is used.
    let checks = fields.iter().chain(exclude.iter()).flatten().map(|field| {
        let mut variant = crate::model::utils::ident_pascal(field);
        variant.set_span(field.span());
        quote_spanned! { field.span()=> let _ = #schema_path::Column::#variant; }
    });
    let column_check = quote! { const _: () = { #(#checks)* }; };

    let fields_expr = ident_list_to_expr(fields);
    let exclude_expr = ident_list_to_expr(exclude);

    let model_impls = quote! {
        impl ::runique::forms::FormEntity for #name {
            type Entity = #schema_path::Entity;
        }
        impl #name {
            pub const objects: ::runique::macros::bdd::objects::Objects<#schema_path::Entity> =
                ::runique::macros::bdd::objects::Objects::new();
        }
    };

    let expanded = quote! {
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        #[serde(transparent)]
        pub struct #name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::field::FormAccess for #name {
            ::runique::impl_form_access!(model);
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

        #column_check
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
