use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── Relation Enum ─────────────────────────────────────────────

pub fn generate_relation_enum() -> TokenStream2 {
    quote! {
        #[derive(Copy, Clone, Debug, ::sea_orm::EnumIter, ::sea_orm::DeriveRelation)]
        pub enum Relation {}
    }
}
