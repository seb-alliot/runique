use quote::{quote, ToTokens};
use syn::{Data, Fields, Field, Attribute};

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

// ==================== HELPERS POUR DeriveModelForm ====================

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

/// Générer la validation pour un champ
pub(crate) fn generate_validation(field: &Field) -> proc_macro2::TokenStream {
    let field_name = field.ident.as_ref().unwrap();
    let field_name_str = field_name.to_string();
    let field_type = infer_field_type(field);

    if is_optional_field(field) {
        quote! {
            self.optional(#field_name_str, &#field_type, raw_data);
        }
    } else {
        quote! {
            self.require(#field_name_str, &#field_type, raw_data);
        }
    }
}

/// Inférer le type de champ
pub(crate) fn infer_field_type(field: &Field) -> proc_macro2::TokenStream {
    let name = field.ident.as_ref().unwrap().to_string();
    let ty = &field.ty;
    let ty_str = quote!(#ty).to_string();

    // Détection par nom
    if name.contains("email") {
        return quote! { rusti::formulaire::field::EmailField };
    }
    if name.contains("password") {
        return quote! { rusti::formulaire::field::PasswordField };
    }
    if name.contains("slug") {
        return quote! { rusti::formulaire::field::SlugField };
    }

    // Détection par type
    let base_type = ty_str.replace("Option < ", "").replace(" >", "").trim().to_string();
    match base_type.as_str() {
        "String" => quote! { rusti::formulaire::field::CharField { allow_blank: false } },
        "i32" | "i64" => quote! { rusti::formulaire::field::IntegerField },
        "f32" | "f64" => quote! { rusti::formulaire::field::FloatField },
        "bool" => quote! { rusti::formulaire::field::BooleanField },
        "NaiveDate" => quote! { rusti::formulaire::field::DateField },
        "DateTime" => quote! { rusti::formulaire::field::DateTimeField },
        _ => quote! { rusti::formulaire::field::CharField { allow_blank: false } },
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

    if is_optional_field(field) {
        quote! {
            #field_name: Set(self.get_value(#field_name_str).ok()),
        }
    } else {
        quote! {
            #field_name: Set(self.get_value(#field_name_str).unwrap()),
        }
    }
}