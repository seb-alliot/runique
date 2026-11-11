//! Form field types — text, number, boolean, choice, datetime, file, media, binary, hidden, special.

// Defined before the modules below so they all see it.
/// What every field type repeats: access to its `base` config and, unless the
/// field is never shown, the `label` builder.
macro_rules! field_base {
    ($field:ty) => {
        impl $crate::forms::base::CommonFieldConfig for $field {
            fn get_field_config(&self) -> &$crate::forms::base::FieldConfig {
                &self.base
            }

            fn get_field_config_mut(&mut self) -> &mut $crate::forms::base::FieldConfig {
                &mut self.base
            }
        }
    };
    ($field:ty, label) => {
        field_base!($field);

        impl $field {
            /// Overrides the auto-generated label.
            pub fn label(mut self, label: &str) -> Self {
                self.base.label = label.to_string();
                self
            }
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
