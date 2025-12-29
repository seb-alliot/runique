use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};
use crate::helpers::*;

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

    let validations: Vec<_> = fields.iter()
        .filter(|f| !is_excluded(f))
        .map(|f| generate_validation(f))
        .collect();

    let conversions: Vec<_> = fields.iter()
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

        impl ::rusti::formulaire::formsrusti::FormulaireTrait for #form_name {
            fn new() -> Self {
                Self { form: ::rusti::formulaire::formsrusti::Forms::new() }
            }

            fn validate(&mut self, raw_data: &std::collections::HashMap<String, String>) -> bool {
                #(#validations)*
                self.is_valid()
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