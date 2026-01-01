use crate::helpers::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

/// Implémentation de la macro #[rusti_form]
pub(crate) fn rusti_form_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    // 1. Ajouter #[serde(flatten)] sur le champ Forms
    if let Data::Struct(ref mut data_struct) = input.data {
        if let Fields::Named(ref mut fields) = data_struct.fields {
            for field in fields.named.iter_mut() {
                if is_forms_field(field) {
                    add_serde_flatten(field);
                }
            }
        }
    }

    let form_field = find_forms_field(&input.data, name);

    let has_derive = has_derive_attribute(&input.attrs);

    let derive_clause = if has_derive {
        quote! {}
    } else {
        quote! {
            #[derive(::rusti::serde::Serialize, ::rusti::serde::Deserialize, Debug)]
        }
    };

    let expanded = quote! {
        #derive_clause
        #input

        impl std::ops::Deref for #name {
            type Target = ::rusti::formulaire::formsrusti::Forms;
            fn deref(&self) -> &Self::Target { &self.#form_field }
        }

        impl std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.#form_field }
        }
    };

    TokenStream::from(expanded)
}
