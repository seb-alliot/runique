use proc_macro::TokenStream;

mod model;
mod schema_form;

/// Macro #[form(...)]
#[proc_macro_attribute]
pub fn form(attr: TokenStream, item: TokenStream) -> TokenStream {
    schema_form::model_schema(attr, item)
}

#[proc_macro]
pub fn model(input: TokenStream) -> TokenStream {
    model::model_impl(input)
}
