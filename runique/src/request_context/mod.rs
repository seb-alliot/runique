// --- Sous-modules ---
pub mod composant_request;
pub mod processor;
pub mod request_struct;
pub mod template_context;
pub mod tera_tool;

// --- Ré-exports pour simplifier l'accès depuis l'extérieur ---
pub use composant_request::{FlashManager, RuniqueContext};
pub use processor::{FlashMessageData, Message};
pub use template_context::TemplateContext;
