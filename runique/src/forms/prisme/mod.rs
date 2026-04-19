//! Prisme pipeline — sentinel (access) → CSRF → parsing.
pub mod aegis;
pub mod csrf_gate;
pub mod rules;
pub mod sentinel;

pub use aegis::aegis;
pub use rules::*;
pub use sentinel::sentinel;
