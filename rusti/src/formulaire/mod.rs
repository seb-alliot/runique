pub mod sanetizer;

pub use sanetizer::{
    auto_sanitize,
    is_dangerous,
    is_sensitive_field,
};

pub mod field;
pub use field::{
    RustiField,
    CharField,
    TextField,
    PasswordField,
    EmailField,
    IntegerField,
    FloatField,
    BooleanField,
    DateField,
    DateTimeField,
    IPAddressField,
    URLField,
    SlugField,
    JSONField,
};

pub mod formsrusti;
pub use formsrusti::{Forms, FormulaireTrait};

pub mod extracteur;
pub use extracteur::ExtractForm;