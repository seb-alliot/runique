//! Macro `define_enum_kind!` — generates the `FieldKind` enum with one variant per field type.

/// Generates the `FieldKind` enum with one variant per registered field type,
/// plus a `From<$field_type> for GenericField` impl for each variant so any
/// field type can be wrapped into a `GenericField` with `.into()`.
#[macro_export]
macro_rules! define_enum_kind {
    (
        $(
            $(#[$attr:meta])*
            $variant:ident => $field_type:ty
        ),* $(,)?
    ) => {
        #[derive(Clone, Debug, Serialize)]
        pub enum FieldKind {
            $(
                $(#[$attr])*
                $variant($field_type),
            )*
        }

        $(
            impl From<$field_type> for GenericField {
                fn from(f: $field_type) -> Self {
                    GenericField { kind: FieldKind::$variant(f) }
                }
            }
        )*
    };
}
