use crate::model::ast::{FieldDef, FieldOption, FieldType, ModelInput};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

// ── ActiveModel ───────────────────────────────────────────────
pub fn generate_active_model(model: &ModelInput) -> TokenStream2 {
    let behavior = generate_active_model_behavior(&model.fields);
    quote! {
        #behavior
        ::runique::impl_objects!(Entity);
    }
}

/// `ActiveModelBehavior` for a model's entity: fills its `auto_now` columns on
/// insert (unless the row already sets them) and its `auto_now_update` columns
/// on every save — in SeaORM, the same on every engine, instead of a database
/// trigger only some engines had.
pub fn generate_active_model_behavior(fields: &[FieldDef]) -> TokenStream2 {
    let stamped = |field: &FieldDef, wanted: fn(&FieldOption) -> bool| {
        field
            .options
            .iter()
            .any(wanted)
            .then(|| now_value(&field.column_type()))?
    };
    let on_insert: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let now = stamped(f, |o| matches!(o, FieldOption::AutoNow))?;
            let name = &f.name;
            Some(quote! {
                if insert && ::sea_orm::ActiveValue::is_not_set(&self.#name) {
                    self.#name = ::sea_orm::ActiveValue::Set(::std::option::Option::Some(#now));
                }
            })
        })
        .collect();
    let on_save: Vec<TokenStream2> = fields
        .iter()
        .filter_map(|f| {
            let now = stamped(f, |o| matches!(o, FieldOption::AutoNowUpdate))?;
            let name = &f.name;
            Some(quote! {
                self.#name = ::sea_orm::ActiveValue::Set(::std::option::Option::Some(#now));
            })
        })
        .collect();

    if on_insert.is_empty() && on_save.is_empty() {
        return quote! { impl ::sea_orm::ActiveModelBehavior for ActiveModel {} };
    }
    quote! {
        #[::runique::prelude::async_trait]
        impl ::sea_orm::ActiveModelBehavior for ActiveModel {
            async fn before_save<C>(
                mut self,
                _db: &C,
                insert: bool,
            ) -> ::std::result::Result<Self, ::sea_orm::DbErr>
            where
                C: ::sea_orm::ConnectionTrait,
            {
                #(#on_insert)*
                #(#on_save)*
                ::std::result::Result::Ok(self)
            }
        }
    }
}

/// The current time in the column's Rust type; `None` for a column that
/// can't hold a timestamp.
fn now_value(ty: &FieldType) -> Option<TokenStream2> {
    match ty {
        FieldType::Datetime | FieldType::Timestamp => {
            Some(quote! { ::chrono::Utc::now().naive_utc() })
        }
        FieldType::TimestampTz => Some(quote! { ::chrono::Utc::now() }),
        FieldType::Date => Some(quote! { ::chrono::Utc::now().date_naive() }),
        FieldType::Time => Some(quote! { ::chrono::Utc::now().time() }),
        _ => None,
    }
}
