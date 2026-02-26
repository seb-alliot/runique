use proc_macro::TokenStream;

// Modules PRIVÉS (pas de pub)
mod forms;
mod helpers;
mod model;
mod models;

/// Macro #[runique_form]
#[proc_macro_attribute]
pub fn runique_form(attr: TokenStream, item: TokenStream) -> TokenStream {
    forms::runique_form_impl(attr, item)
}

/// Macro #[derive(DeriveModelForm)]
#[proc_macro_derive(DeriveModelForm, attributes(model_form, form_field))]
pub fn derive_model_form(input: TokenStream) -> TokenStream {
    models::derive_model_form_impl(input)
}

mod schema_form;

/// Macro #[derive(ModelSchemaForm)]
#[proc_macro_attribute]
pub fn form(attr: TokenStream, item: TokenStream) -> TokenStream {
    schema_form::model_schema(attr, item)
}

#[proc_macro]
pub fn model(input: TokenStream) -> TokenStream {
    model::model_impl(input)
}
