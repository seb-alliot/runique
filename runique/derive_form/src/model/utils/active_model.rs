use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── ActiveModel ───────────────────────────────────────────────
pub fn generate_active_model() -> TokenStream2 {
    quote! {
        impl ::sea_orm::ActiveModelBehavior for ActiveModel {}
        ::runique::impl_objects!(Entity);
    }
}
