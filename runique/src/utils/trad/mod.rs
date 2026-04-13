//! Internationalization — global language selection (`set_lang`), `t()` / `tf()` helpers.
pub mod switch_lang;

pub use switch_lang::{Lang, current_lang, set_lang};
pub use switch_lang::{t, tf};
