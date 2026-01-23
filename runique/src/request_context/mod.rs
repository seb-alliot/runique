// --- Sous-modules ---
pub mod composant_request;
pub mod context_error;
pub mod template_context;
pub mod tera_tool;

// --- Ré-exports pour simplifier l'accès depuis l'extérieur ---
pub use template_context::{AppError, TemplateContext};
