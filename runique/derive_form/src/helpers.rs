use quote::{quote, ToTokens};
use syn::{Attribute, Data, Field, Fields};

// ==================== HELPERS PARTAGÉS ====================

/// Vérifie si un #[derive(...)] existe déjà
pub(crate) fn has_derive_attribute(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("derive"))
}

/// Vérifie si un champ est de type Forms
pub(crate) fn is_forms_field(field: &Field) -> bool {
    if let syn::Type::Path(type_path) = &field.ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Forms";
        }
    }
    false
}

/// Ajoute #[serde(flatten)] à un champ
pub(crate) fn add_serde_flatten(field: &mut Field) {
    let flatten_attr: syn::Attribute = syn::parse_quote! {
        #[serde(flatten)]
    };
    field.attrs.push(flatten_attr);
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

/// Exclure les champs système
pub(crate) fn is_excluded(field: &Field) -> bool {
    let name = field.ident.as_ref().unwrap().to_string();

    if name == "id" || name == "created_at" || name == "updated_at" {
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

/// Générer la validation pour runiqueForm (nouveau)
pub(crate) fn generate_validation_runiqueform(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let field_type = infer_field_type(field);

    if is_optional_field(field) {
        quote! {
            form.optional(#field_name_str, &#field_type, raw_data);
        }
    } else {
        quote! {
            form.require(#field_name_str, &#field_type, raw_data);
        }
    }
}

/// Formater le nom du champ en label lisible
/// Ex: "username" -> "Username", "first_name" -> "First Name"
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

/// Déterminer le type de champ Runique à partir du type Rust (pour register_field)
pub(crate) fn get_field_type(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap().to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string().replace(" ", "");

    // Détection par nom de champ (priorité haute)
    if field_name.contains("email") {
        return quote! { ::runique::formulaire::field::EmailField };
    }
    if field_name.contains("password") || field_name.contains("pwd") {
        return quote! { ::runique::formulaire::field::PasswordField };
    }
    if field_name.contains("url") || field_name.contains("link") || field_name.contains("website") {
        return quote! { ::runique::formulaire::field::URLField };
    }
    if field_name.contains("slug") {
        return quote! { ::runique::formulaire::field::SlugField };
    }

    // Enlever Option<> si présent
    let base_type = ty_str
        .replace("Option<", "")
        .replace(">", "")
        .trim()
        .to_string();

    // Détection par type
    if base_type.contains("String") {
        // Vérifier si c'est un TextField
        if field_name.contains("description")
            || field_name.contains("bio")
            || field_name.contains("content")
            || field_name.contains("text")
            || field_name.contains("message")
        {
            return quote! { ::runique::formulaire::field::TextField::new() };
        }
        return quote! { ::runique::formulaire::field::CharField::new() };
    }

    if base_type.contains("i32") || base_type.contains("i64") {
        return quote! { ::runique::formulaire::field::IntegerField };
    }

    if base_type.contains("f32") || base_type.contains("f64") {
        return quote! { ::runique::formulaire::field::FloatField };
    }

    if base_type.contains("bool") {
        return quote! { ::runique::formulaire::field::BooleanField };
    }

    if base_type.contains("DateTime") || base_type.contains("NaiveDateTime") {
        return quote! { ::runique::formulaire::field::DateTimeField };
    }

    if base_type.contains("NaiveDate") || base_type.contains("Date") {
        return quote! { ::runique::formulaire::field::DateField };
    }

    if base_type.contains("IpAddr") {
        return quote! { ::runique::formulaire::field::IPAddressField };
    }

    if base_type.contains("Value") || base_type.contains("Json") {
        return quote! { ::runique::formulaire::field::JSONField };
    }

    // Par défaut: CharField
    quote! { ::runique::formulaire::field::CharField::new() }
}

/// Inférer le type de champ (ancien système)
pub(crate) fn infer_field_type(field: &Field) -> proc_macro2::TokenStream {
    let name = field.ident.as_ref().unwrap().to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string();

    // Détection par nom
    if name.contains("email") {
        return quote! { runique::formulaire::field::EmailField };
    }
    if name.contains("password") {
        return quote! { runique::formulaire::field::PasswordField };
    }
    if name.contains("slug") {
        return quote! { runique::formulaire::field::SlugField };
    }

    // Détection par type
    let base_type = ty_str
        .replace("Option < ", "")
        .replace(" >", "")
        .trim()
        .to_string();
    match base_type.as_str() {
        "String" => quote! { runique::formulaire::field::CharField { allow_blank: false } },
        "i32" | "i64" => quote! { runique::formulaire::field::IntegerField },
        "f32" | "f64" => quote! { runique::formulaire::field::FloatField },
        "bool" => quote! { runique::formulaire::field::BooleanField },
        "NaiveDate" => quote! { runique::formulaire::field::DateField },
        "DateTime" => quote! { runique::formulaire::field::DateTimeField },
        _ => quote! { runique::formulaire::field::CharField { allow_blank: false } },
    }
}

/// Vérifier si Option<T>
pub(crate) fn is_optional_field(field: &Field) -> bool {
    let ty_str = quote!(#field.ty).to_string();
    ty_str.starts_with("Option")
}

/// Générer la conversion
pub(crate) fn generate_conversion(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string().replace(" ", "");

    // Gérer les timestamps auto-générés
    if field_name_str == "created_at" || field_name_str == "updated_at" {
        return quote! {
            #field_name: Set(chrono::Utc::now().naive_utc()),
        };
    }

    // Gérer Option<T>
    if is_optional_field(field) {
        return quote! {
            #field_name: Set(self.get_value(#field_name_str)),
        };
    }

    // Types standards
    let base_type = ty_str
        .replace("Option<", "")
        .replace(">", "")
        .trim()
        .to_string();

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
