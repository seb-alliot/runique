//! Internationalisation — sélection de langue globale (`set_lang`), helpers `t()` / `tf()`.
pub mod switch_lang;

pub use switch_lang::{Lang, current_lang, set_lang};
pub use switch_lang::{t, tf};
