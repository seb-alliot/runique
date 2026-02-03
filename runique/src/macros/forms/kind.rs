#[macro_export]
macro_rules! delegate_to_kind {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match &$self.kind {
            $crate::forms::generic::FieldKind::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::File(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Boolean(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Choice(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Radio(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Checkbox(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Date(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Time(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::DateTime(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Duration(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Color(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Slug(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::UUID(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::JSON(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::IPAddress(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Hidden(field) => field.$method($($arg),*),
        }
    };

    (mut $self:expr, $method:ident $(, $arg:expr)*) => {
        match &mut $self.kind {
            $crate::forms::generic::FieldKind::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::File(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Boolean(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Choice(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Radio(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Checkbox(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Date(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Time(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::DateTime(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Duration(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Color(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Slug(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::UUID(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::JSON(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::IPAddress(field) => field.$method($($arg),*),
            $crate::forms::generic::FieldKind::Hidden(field) => field.$method($($arg),*),
        }
    };
}
