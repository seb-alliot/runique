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
    IPAddressField,
    DateField,
    DateTimeField,
    JSONField,
};


pub mod forms_rusti;
pub use forms_rusti::Forms;

pub mod extracteur;
pub use extracteur::AxumForm;