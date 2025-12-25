use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields, Field, Attribute};

#[proc_macro_attribute]
pub fn rusti_form(_attr: TokenStream, item: TokenStream) -> TokenStream {
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

    let form_field = find_forms_field(&input.data, &name);

    // 2. Ajouter les derives si pas déjà présents
    let has_derive = has_derive_attribute(&input.attrs);

    let derive_clause = if has_derive {
        quote! {}  // Rien si déjà présent
    } else {
        quote! {
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
        }
    };

    let expanded = quote! {
        #derive_clause
        #input

        impl std::ops::Deref for #name {
            type Target = rusti::formulaire::forms_rusti::Forms;
            fn deref(&self) -> &Self::Target { &self.#form_field }
        }

        impl std::ops::DerefMut for #name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.#form_field }
        }
    };

    TokenStream::from(expanded)
}

/// Vérifie si un #[derive(...)] existe déjà
fn has_derive_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        attr.path().is_ident("derive")
    })
}

fn is_forms_field(field: &Field) -> bool {
    if let syn::Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Forms";
        }
    }
    false
}

fn add_serde_flatten(field: &mut Field) {
    let flatten_attr: syn::Attribute = syn::parse_quote! {
        #[serde(flatten)]
    };
    field.attrs.push(flatten_attr);
}

fn find_forms_field(data: &Data, struct_name: &syn::Ident) -> syn::Ident {
    let fields = match data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("Structs à champs nommés uniquement"),
        },
        _ => panic!("Fonctionne uniquement sur des structs"),
    };

    for field in fields {
        if is_forms_field(field) {
            return field.ident.clone().expect("Champ sans nom");
        }
    }

    panic!("Struct '{}' doit avoir un champ 'Forms'", struct_name)
}