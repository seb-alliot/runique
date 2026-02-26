pub mod ast;
pub mod generateur;
pub mod parser;
pub mod utils;

pub use ast::*;

pub(crate) fn model_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let model = syn::parse_macro_input!(input as ast::ModelInput);
    let generated = generateur::generate(&model);
    proc_macro::TokenStream::from(generated)
}
