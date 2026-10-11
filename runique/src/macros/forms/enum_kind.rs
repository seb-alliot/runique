//! Macro `define_enum_field!` — generates the `Field` enum with one variant per field type.

/// Generates the `Field` enum with one variant per registered field type,
/// plus a `From<$field_type> for GenericField` impl for each variant so any
/// field type can be wrapped into a `GenericField` with `.into()`.
#[macro_export]
macro_rules! define_enum_field {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $field_type:ty
        ),* $(,)?
    ) => {
        #[derive(Clone, Debug, Serialize)]
        pub enum GeneratedField {
            $(
                $(#[$attr])*
                $variant($field_type),
            )*
        }

        $(
            impl From<$field_type> for GenericField {
                fn from(f: $field_type) -> Self {
                    GenericField { field: GeneratedField::$variant(f) }
                }
            }
        )*
    };
}
