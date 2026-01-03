pub mod sanetizer;

pub use sanetizer::{auto_sanitize, is_dangerous, is_sensitive_field};

pub mod field;
pub use field::{
    BooleanField, CharField, DateField, DateTimeField, EmailField, FloatField, IPAddressField,
    IntegerField, JSONField, PasswordField, RuniqueField, SlugField, TextField, URLField,
};

pub mod formsrunique;
pub use formsrunique::{Forms, RuniqueForm};

pub mod extracteur;
pub use extracteur::ExtractForm;
