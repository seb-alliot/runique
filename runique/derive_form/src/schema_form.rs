use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Path};

pub(crate) fn model_schema(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // Parser les attributs : schema = fn_name, fields = [a, b, c]
    let attr2: TokenStream2 = attr.into();
    let attr_str = attr2.to_string();

    // Extraire schema
    let schema_fn =
        extract_value(&attr_str, "schema").expect("model_schema_form : schema = fn_name requis");
    let schema_path: Path =
        syn::parse_str(&schema_fn).expect("model_schema_form : chemin invalide");

    // Extraire fields optionnel
    let fields_expr = match extract_list(&attr_str, "fields") {
        Some(fields) => {
            let lits: Vec<syn::LitStr> = fields
                .iter()
                .map(|f| syn::LitStr::new(f, proc_macro2::Span::call_site()))
                .collect();
            quote! { Some(&[#(#lits),*]) }
        }
        None => quote! { None },
    };

    // Extraire exclude optionnel
    let exclude_expr = match extract_list(&attr_str, "exclude") {
        Some(fields) => {
            let lits: Vec<syn::LitStr> = fields
                .iter()
                .map(|f| syn::LitStr::new(f, proc_macro2::Span::call_site()))
                .collect();
            quote! { Some(&[#(#lits),*]) }
        }
        None => quote! { None },
    };

    let expanded = quote! {
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        #[serde(transparent)]
        pub struct #name {
            pub form: ::runique::forms::Forms,
        }

        impl ::runique::forms::model_form::ModelForm for #name {
            fn schema() -> ::runique::migration::schema::ModelSchema {
                #schema_path()
            }
            fn fields() -> Option<&'static [&'static str]> {
                #fields_expr
            }
            fn exclude() -> Option<&'static [&'static str]> {
                #exclude_expr
            }
        }

        impl ::runique::forms::field::RuniqueForm for #name {
            fn register_fields(form: &mut ::runique::forms::Forms) {
                <Self as ::runique::forms::model_form::ModelForm>::model_register_fields(form);
            }
            fn from_form(form: ::runique::forms::Forms) -> Self {
                Self { form }
            }
            fn get_form(&self) -> &::runique::forms::Forms {
                &self.form
            }
            fn get_form_mut(&mut self) -> &mut ::runique::forms::Forms {
                &mut self.form
            }
        }
    };

    TokenStream::from(expanded)
}

fn extract_value(attr_str: &str, key: &str) -> Option<String> {
    let pattern = format!("{} =", key);
    let start = attr_str.find(&pattern)? + pattern.len();
    let rest = attr_str[start..].trim();
    let end = rest
        .find(|c| [',', ']', ')'].contains(&c))
        .unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

fn extract_list(attr_str: &str, key: &str) -> Option<Vec<String>> {
    let pattern = format!("{} = [", key);
    let start = attr_str.find(&pattern)? + pattern.len();
    let end = attr_str[start..].find(']')?;
    let list_str = &attr_str[start..start + end];
    let items: Vec<String> = list_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if items.is_empty() {
        None
    } else {
        Some(items)
    }
}
