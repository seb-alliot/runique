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
        .map(|f| generate_validation_rustiform(f))
        .collect();

    // Générer les conversions pour to_active_model
    let conversions: Vec<_> = fields
        .iter()
        .filter(|f| !is_excluded(f))
        .map(|f| generate_conversion(f))
        .collect();

    let expanded = quote! {
        #[derive(::rusti::serde::Serialize, ::rusti::serde::Deserialize, Debug)]
        pub struct #form_name {
            #[serde(flatten)]
            pub form: ::rusti::formulaire::formsrusti::Forms,
        }

        impl std::ops::Deref for #form_name {
            type Target = ::rusti::formulaire::formsrusti::Forms;
            fn deref(&self) -> &Self::Target { &self.form }
        }

        impl std::ops::DerefMut for #form_name {
            fn deref_mut(&mut self) -> &mut Self::Target { &mut self.form }
        }

        // Nouveau trait RustiForm
        impl ::rusti::formulaire::formsrusti::RustiForm for #form_name {
            fn register_fields(form: &mut ::rusti::formulaire::formsrusti::Forms) {
                #(#register_fields)*
            }

            fn validate_fields(form: &mut ::rusti::formulaire::formsrusti::Forms, raw_data: &std::collections::HashMap<String, String>) {
                #(#validations)*
            }

            fn from_form(form: ::rusti::formulaire::formsrusti::Forms) -> Self {
                Self { form }
            }

            fn get_form(&self) -> &::rusti::formulaire::formsrusti::Forms {
                &self.form
            }

            fn get_form_mut(&mut self) -> &mut ::rusti::formulaire::formsrusti::Forms {
                &mut self.form
            }
        }

        impl #form_name {
            pub fn to_active_model(&self) -> ActiveModel {
                use ::rusti::sea_orm::ActiveValue::Set;
                ActiveModel {
                    #(#conversions)*
                    ..Default::default()
                }
            }

            pub async fn save(&self, db: &::rusti::sea_orm::DatabaseConnection)
                -> Result<#model_name, ::rusti::sea_orm::DbErr>
            {
                use ::rusti::sea_orm::EntityTrait;
                self.to_active_model().insert(db).await
            }
        }
    };

    TokenStream::from(expanded)
}
