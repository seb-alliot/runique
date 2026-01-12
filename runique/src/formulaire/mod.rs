pub mod composant_field;
pub mod extracteur;
pub mod field;
pub mod formsrunique;
pub mod sanetizer;
pub use sanetizer::{auto_sanitize, is_dangerous, is_sensitive_field};

// Ré-exportations pour un usage plus simple
pub use extracteur::ExtractForm;
pub use formsrunique::RuniqueForm;

// pub use field::RuniqueField;
