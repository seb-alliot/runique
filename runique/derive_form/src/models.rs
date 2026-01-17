use crate::helpers::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub(crate) fn derive_model_form_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let model_name = &input.ident;
    let form_name = quote::format_ident!("{}Form", model_name);

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => panic!("DeriveModelForm : structs avec champs nommés uniquement"),
        },
        _ => panic!("DeriveModelForm : structs uniquement"),
    };

    // Générer les champs pour register_fields
    let register_fields: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(|f| {
            let field_name = f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let label = format_field_label(&field_name_str);

            let field_constructor = get_field_type(f);
            let is_optional = is_optional_field(f);

            let required_clause = if is_optional {
                quote! {}
            } else {
                quote! { .required("Ce champ est obligatoire") }
            };

            quote! {
                form.field(
                    &#field_constructor
                        .label(#label)
                        #required_clause
                );
            }
        })
        .collect();

    // Générer les conversions pour to_active_model
    let conversions: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(generate_conversion)
        .collect();

    let expanded = quote! {
        #[derive(::runique::serde::Serialize, Debug, Clone)]
        pub struct #form_name {
            #[serde(flatten, skip_deserializing)]
            pub form: ::runique::formulaire::builder_form::form_manager::Forms,
        }

        // Implémentation manuelle de Deserialize
        impl<'de> ::runique::serde::Deserialize<'de> for #form_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::runique::serde::Deserializer<'de>,
            {
                // On désérialise juste un objet vide
                let _ = <std::collections::HashMap<String, ::runique::serde_json::Value>>::deserialize(deserializer)?;

                // On retourne un formulaire vide qui sera reconstruit par build()
                Ok(Self {
                    form: ::runique::formulaire::builder_form::form_manager::Forms::new(),
                })
            }
        }

        impl std::ops::Deref for #form_name {
            type Target = ::runique::formulaire::builder_form::form_manager::Forms;
            fn deref(&self) -> &Self::Target {
                &self.form
            }
        }

        impl std::ops::DerefMut for #form_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.form
            }
        }

        impl ::runique::formulaire::builder_form::trait_form::RuniqueForm for #form_name {
            fn register_fields(form: &mut ::runique::formulaire::builder_form::form_manager::Forms) {
                #(#register_fields)*
            }

            fn from_form(form: ::runique::formulaire::builder_form::form_manager::Forms) -> Self {
                Self { form }
            }

            fn get_form(&self) -> &::runique::formulaire::builder_form::form_manager::Forms {
                &self.form
            }

            fn get_form_mut(&mut self) -> &mut ::runique::formulaire::builder_form::form_manager::Forms {
                &mut self.form
            }
        }

        impl #form_name {
            pub fn to_active_model(&self) -> ActiveModel {
                use ::runique::sea_orm::ActiveValue::Set;
                ActiveModel {
                    #(#conversions)*
                    ..Default::default()
                }
            }

            pub async fn save(&self, db: &::runique::sea_orm::DatabaseConnection)
                -> Result<#model_name, ::runique::sea_orm::DbErr>
            {
                use ::runique::sea_orm::EntityTrait;
                self.to_active_model().insert(db).await
            }
        }
    };
    TokenStream::from(expanded)
}
