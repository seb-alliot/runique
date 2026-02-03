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
