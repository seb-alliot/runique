use darling::FromField;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{Attribute, Data, Field, Fields, Type};

/// Vérifie si un #[derive(...)] existe déjà
pub(crate) fn has_derive_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("derive"))
}

/// Vérifie si un champ est de type Forms
pub(crate) fn is_forms_field(field: &Field) -> bool {
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Forms";
        }
    }
    false
}

/// Ajoute #[serde(flatten)] à un champ
pub(crate) fn add_serde_flatten(field: &mut Field) {
    let attr: syn::Attribute = syn::parse_quote! {
        #[serde(flatten, skip_deserializing, default)]
    };
    field.attrs.push(attr);
}

/// Trouve le champ de type Forms
pub(crate) fn find_forms_field(data: &Data, struct_name: &syn::Ident) -> syn::Ident {
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

pub(crate) fn is_excluded(field: &Field) -> bool {
    let name = field.ident.as_ref().unwrap().to_string();

    if name == "id"
        || name == "csrf_token"
        || name == "_csrf_token"
        || name == "form"
        || name == "created_at"
        || name == "updated_at"
        || name == "is_active"
        || name == "deleted_at"
    {
        return true;
    }

    for attr in &field.attrs {
        if attr.path().is_ident("sea_orm") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("primary_key") {
                return true;
            }
        }
    }
    false
}

/// Vérifier si Option<T>
pub(crate) fn is_optional_field(field: &Field) -> bool {
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Déterminer le type de champ field à partir du type Rust
/// Déterminer le type de champ field à partir du type Rust
pub(crate) fn get_field_type(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string().replace(" ", "");
    let base_type = ty_str.replace("Option<", "").replace(">", "");

    // 1. Détection par nom (Module text_mode)
    if field_name_str.contains("email") {
        return quote! {
            ::runique::forms::fields::TextField::email(#field_name_str)
        };
    }
    if field_name_str.contains("password") || field_name_str.contains("pwd") {
        return quote! {
            ::runique::forms::fields::TextField::password(#field_name_str)
        };
    }
    if field_name_str.contains("url")
        || field_name_str.contains("link")
        || field_name_str.contains("website")
    {
        return quote! {
            ::runique::forms::fields::TextField::url(#field_name_str)
        };
    }

    // 2. Détection par type
    if base_type.contains("String") {
        if field_name_str.contains("description")
            || field_name_str.contains("bio")
            || field_name_str.contains("content")
            || field_name_str.contains("message")
        {
            return quote! {
                ::runique::forms::fields::TextField::textarea(#field_name_str)
            };
        }
        return quote! {
            ::runique::forms::fields::TextField::text(#field_name_str)
        };
    }

    // Types numériques (Module number_mode)
    if base_type.contains("i32") || base_type.contains("i64") || base_type.contains("u32") {
        return quote! {
            ::runique::forms::fields::NumericField::integer(#field_name_str)
        };
    }

    if base_type.contains("f32") || base_type.contains("f64") {
        return quote! {
            ::runique::forms::fields::NumericField::float(#field_name_str)
        };
    }

    // Fallback standard
    quote! {
        ::runique::forms::fields::TextField::text(#field_name_str)
    }
}

/// Formater le nom du champ en label
pub(crate) fn format_field_label(field_name: &str) -> String {
    field_name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Générer la conversion vers SeaORM
pub(crate) fn generate_conversion(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string().replace(" ", "");

    // Timestamps automatiques
    if field_name_str == "created_at" || field_name_str == "updated_at" {
        return quote! {
            #field_name: Set(chrono::Utc::now().naive_utc()),
        };
    }
    // Exemple pour is_active avec valeur par défaut
    if field_name_str == "is_active" {
        // Chercher l'attribut #[model_form(default = ...)]
        let default_value = get_default_value(field);
        return quote! {
            #field_name: Set(#default_value),
        };
    }
    // Option<T>
    if is_optional_field(field) {
        return quote! {
            #field_name: Set(self.form.get_option(#field_name_str)),
        };
    }

    let base_type = ty_str.replace("Option<", "").replace(">", "");

    // Types numériques
    if base_type.contains("i32") {
        quote! {
            #field_name: Set(self.form.get_i32(#field_name_str)),
        }
    } else if base_type.contains("i64") {
        quote! {
            #field_name: Set(self.form.get_i64(#field_name_str)),
        }
    } else if base_type.contains("u32") {
        quote! {
            #field_name: Set(self.form.get_u32(#field_name_str)),
        }
    } else if base_type.contains("u64") {
        quote! {
            #field_name: Set(self.form.get_u64(#field_name_str)),
        }
    } else if base_type.contains("f32") {
        quote! {
            #field_name: Set(self.form.get_f32(#field_name_str)),
        }
    } else if base_type.contains("f64") {
        quote! {
            #field_name: Set(self.form.get_f64(#field_name_str)),
        }
    } else if base_type.contains("bool") {
        quote! {
            #field_name: Set(self.form.get_bool(#field_name_str)),
        }
    } else {
        // String par défaut
        quote! {
            #field_name: Set(self.form.get_string(#field_name_str)),
        }
    }
}

#[derive(FromField)]
#[darling(attributes(model_form))]
struct FieldArgs {
    #[darling(default)]
    default: bool,
}

pub(crate) fn get_default_value(field: &Field) -> TokenStream {
    match FieldArgs::from_field(field) {
        Ok(args) => {
            let val = args.default;
            quote! { #val }
        }
        Err(_) => quote! { true },
    }
}
