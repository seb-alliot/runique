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

    if name == "id" ||
       name == "csrf_token" ||
       name == "_csrf_token" ||
       name == "form" ||
       name == "created_at" ||
       name == "updated_at" {
        return true;
    }

    // Exclusion des clés primaires SeaORM
    for attr in &field.attrs {
        if attr.path().is_ident("sea_orm") {
            let tokens = attr.meta.to_token_stream().to_string();
            if tokens.contains("primary_key") { return true; }
        }
    }
    false
}

/// Vérifier si Option<T> est utilisé
pub(crate) fn is_optional_field(field: &Field) -> bool {
    if let Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.first() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Déterminer le type de champ Runique à partir du type Rust
pub(crate) fn get_field_type(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap().to_string().to_lowercase();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string().replace(" ", "");

    // 1. Détection par nom (priorité haute)
    if field_name.contains("email") {
        return quote! { EmailField };
    }
    if field_name.contains("password") || field_name.contains("pwd") {
        return quote! { PasswordField };
    }
    if field_name.contains("url") || field_name.contains("link") || field_name.contains("website") {
        return quote! { URLField };
    }
    if field_name.contains("slug") {
        return quote! { SlugField };
    }

    // 2. Détection par type
    let base_type = ty_str.replace("Option<", "").replace(">", "");

    if base_type.contains("String") {
        if field_name.contains("description")
            || field_name.contains("bio")
            || field_name.contains("content")
            || field_name.contains("text")
            || field_name.contains("message")
        {
            return quote! { TextareaField };
        }
        return quote! { CharField };
    }

    if base_type.contains("i32") || base_type.contains("i64") {
        return quote! { IntegerField };
    }

    if base_type.contains("f32") || base_type.contains("f64") {
        return quote! { FloatField };
    }

    if base_type.contains("bool") {
        return quote! { BooleanField };
    }

    if base_type.contains("DateTime") || base_type.contains("NaiveDateTime") {
        return quote! { DateTimeField };
    }

    if base_type.contains("NaiveDate") || base_type.contains("Date") {
        return quote! { DateField };
    }

    if base_type.contains("IpAddr") {
        return quote! { IPAddressField };
    }

    if base_type.contains("Value") || base_type.contains("Json") {
        return quote! { JSONField };
    }

    // Fallback propre
    quote! { CharField }
}


/// Générer la validation pour RuniqueForm
pub(crate) fn generate_validation_runiqueform(field: &Field) -> proc_macro2::TokenStream {
    let field_name_str = field.ident.as_ref().unwrap().to_string();
    let field_type = get_field_type(field);

    if is_optional_field(field) {
        quote! {
            form.optional(#field_name_str, &::runique::prelude::#field_type::new(), raw_data);
        }
    } else {
        quote! {
            form.require(#field_name_str, &::runique::prelude::#field_type::new(), raw_data);
        }
    }
}

/// Formater le nom du champ en label lisible
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

    // Champ interne ignoré
    if field_name_str == "_csrf_token" {
        return quote! {};
    }

    // Timestamps automatiques
    if field_name_str == "created_at" || field_name_str == "updated_at" {
        return quote! {
            #field_name: Set(chrono::Utc::now().naive_utc()),
        };
    }

    // Option<T>
    if is_optional_field(field) {
        return quote! {
            #field_name: Set(self.get_value(#field_name_str)),
        };
    }

    let base_type = ty_str.replace("Option<", "").replace(">", "");

    if base_type.contains("i32") {
        quote! {
            #field_name: Set(self.get_value::<i64>(#field_name_str).unwrap_or(0) as i32),
        }
    } else if base_type.contains("i64") {
        quote! {
            #field_name: Set(self.get_value::<i64>(#field_name_str).unwrap_or(0)),
        }
    } else if base_type.contains("f32") {
        quote! {
            #field_name: Set(self.get_value::<f64>(#field_name_str).unwrap_or(0.0) as f32),
        }
    } else if base_type.contains("f64") {
        quote! {
            #field_name: Set(self.get_value::<f64>(#field_name_str).unwrap_or(0.0)),
        }
    } else if base_type.contains("bool") {
        quote! {
            #field_name: Set(self.get_value::<bool>(#field_name_str).unwrap_or(false)),
        }
    } else {
        quote! {
            #field_name: Set(self.get_value(#field_name_str).unwrap_or_default()),
        }
    }
}
