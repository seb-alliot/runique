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

    // Générer les register_field pour chaque champ
    let register_fields: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(|f| {
            let field_name = &f.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let label = format_field_label(&field_name_str);
            let field_type = get_field_type(f);

            quote! {
                form.register_field(#field_name_str, #label, &#field_type);
            }
        })
        .collect();

    // Générer les validations pour chaque champ
    let validations: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(generate_validation_runiqueform)
        .collect();

    // Générer les conversions pour to_active_model
    let conversions: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(generate_conversion)
        .collect();

    let expanded = quote! {
        #[derive(::runique::serde::Serialize, ::runique::serde::Deserialize, Debug)]
        pub struct #form_name {
            #[serde(flatten)]
            pub form: ::runique::formulaire::formsrunique::Forms,
        }

        impl std::ops::Deref for #form_name {
            type Target = ::runique::formulaire::formsrunique::Forms;
            fn deref(&self) -> &Self::Target { &self.form }
        }

        impl std::ops::DerefMut for #form_name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.form }
        }

        // Nouveau trait RuniqueForm
        impl ::runique::formulaire::formsrunique::RuniqueForm for #form_name {
            fn register_fields(form: &mut ::runique::formulaire::formsrunique::Forms) {
                #(#register_fields)*
            }

            fn validate_fields(form: &mut ::runique::formulaire::formsrunique::Forms, raw_data: &std::collections::HashMap<String, String>) {
                #(#validations)*
            }

            fn from_form(form: ::runique::formulaire::formsrunique::Forms) -> Self {
                Self { form }
            }

            fn get_form(&self) -> &::runique::formulaire::formsrunique::Forms {
                &self.form
            }

            fn get_form_mut(&mut self) -> &mut ::runique::formulaire::formsrunique::Forms {
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
