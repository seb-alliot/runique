//! Internationalization of the Runique framework: global language selection
//! (`set_lang`), `t()` / `tf()` helpers, and the messages in 9 languages.
//!
//! Used through `runique::utils::trad`, which re-exports this crate.
pub mod switch_lang;

pub use switch_lang::{Lang, current_lang, set_lang};
pub use switch_lang::{t, tf};
