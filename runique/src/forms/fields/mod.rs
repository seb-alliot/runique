//! Form field types — text, number, boolean, choice, datetime, file, media, binary, hidden, special.

// Defined before the modules below so they all see it.
/// What every field type repeats: access to its `base` config and, unless the
/// field is never shown, the `label` builder.
macro_rules! field_base {
    (@label) => {
        /// Overrides the auto-generated label.
        pub fn label(mut self, label: &str) -> Self {
            self.base.label = label.to_string();
            self
        }
    };
    (@required) => {
        /// Marks the field as required: left empty, it fails validation.
        pub fn required(mut self) -> Self {
            $crate::forms::traits::form_field::FormField::set_required(&mut self, true, None);
            self
        }
    };
    (@placeholder) => {
        /// Sets the HTML `placeholder` attribute.
        pub fn placeholder(mut self, placeholder: &str) -> Self {
            $crate::forms::traits::form_field::FormField::set_placeholder(&mut self, placeholder);
            self
        }
    };
    ($field:ty) => {
        impl $crate::forms::traits::form_field::FieldAccess for $field {
            fn get_field_config(&self) -> &$crate::forms::config::field_config::FieldConfig {
                &self.base
            }

            fn get_field_config_mut(&mut self) -> &mut $crate::forms::config::field_config::FieldConfig {
                &mut self.base
            }
        }
    };
    ($field:ty, $($builder:ident),+) => {
        field_base!($field);

        impl $field {
            $(field_base!(@$builder);)+
        }
    };
}

pub mod binary;
pub mod boolean;
pub mod choice;
pub mod datetime;
pub mod file;
pub mod hidden;
pub mod media;
pub mod number;
pub mod special;
pub mod text;

pub use binary::*;
pub use boolean::*;
pub use choice::*;
pub use datetime::*;
pub use file::*;
pub use hidden::*;
pub use media::*;
pub use number::*;
pub use special::*;
pub use text::*;
