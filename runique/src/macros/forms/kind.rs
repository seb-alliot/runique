//! Macro `delegate_to_field` — dispatch to the `GeneratedField` variant without repetitive match blocks.

#[macro_export]
macro_rules! delegate_to_field {
    ($self:expr, $method:ident $(, $arg:expr)*) => {
        match &$self.field {
            $crate::forms::generic::GeneratedField::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::File(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Binary(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Boolean(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Choice(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Radio(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Checkbox(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Date(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Time(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::DateTime(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Duration(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Color(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Slug(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::UUID(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::JSON(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::IPAddress(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Hidden(field) => field.$method($($arg),*),
        }
    };

    (mut $self:expr, $method:ident $(, $arg:expr)*) => {
        match &mut $self.field {
            $crate::forms::generic::GeneratedField::Text(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Numeric(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::File(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Binary(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Boolean(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Choice(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Radio(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Checkbox(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Date(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Time(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::DateTime(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Duration(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Color(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Slug(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::UUID(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::JSON(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::IPAddress(field) => field.$method($($arg),*),
            $crate::forms::generic::GeneratedField::Hidden(field) => field.$method($($arg),*),
        }
    };
}
