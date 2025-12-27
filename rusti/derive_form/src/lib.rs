use proc_macro::TokenStream;

// Modules PRIVÉS (pas de pub)
mod forms;
mod models;
mod helpers;

// ==================== MACROS PUBLIQUES ====================

/// Macro #[rusti_form]
#[proc_macro_attribute]
pub fn rusti_form(attr: TokenStream, item: TokenStream) -> TokenStream {
    forms::rusti_form_impl(attr, item) 
}

/// Macro #[derive(DeriveModelForm)]
#[proc_macro_derive(DeriveModelForm, attributes(model_form, form_field))]
pub fn derive_model_form(input: TokenStream) -> TokenStream {
    models::derive_model_form_impl(input)
}