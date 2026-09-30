pub use runique_dsl::ast;
pub mod generateur;
pub mod utils;

pub use ast::*;

pub(crate) fn model_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let model = syn::parse_macro_input!(input as ast::ModelInput);
    if let Err(e) = utils::check_engine_support(&model) {
        return e.to_compile_error().into();
    }
    let generated = generateur::generate(&model);
    proc_macro::TokenStream::from(generated)
}
