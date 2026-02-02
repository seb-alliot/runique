#[macro_export]
macro_rules! delegate_to_kind {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match &$self.kind {
            $crate::forms::generic::FieldKind::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::File(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Boolean(field) => field.$method($($arg),*),
        }
    };

    (mut $self:expr, $method:ident $(, $arg:expr)*) => {
        match &mut $self.kind {
            $crate::forms::generic::FieldKind::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::File(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Boolean(field) => field.$method($($arg),*),
        }
    };
}
