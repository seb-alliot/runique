//! Prisme pipeline — parsing (Aegis) → CSRF check.
pub mod aegis;
pub mod refusal;

pub use aegis::aegis;
pub use refusal::BodyRefusal;
