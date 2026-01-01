pub mod sanetizer;

pub use sanetizer::{auto_sanitize, is_dangerous, is_sensitive_field};

pub mod field;
pub use field::{
    BooleanField, CharField, DateField, DateTimeField, EmailField, FloatField, IPAddressField,
    IntegerField, JSONField, PasswordField, RustiField, SlugField, TextField, URLField,
};

pub mod formsrusti;
pub use formsrusti::{Forms, RustiForm};

pub mod extracteur;
pub use extracteur::ExtractForm;
