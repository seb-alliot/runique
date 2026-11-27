//! Internationalization of the Runique framework: the language in use
//! (`set_lang`), `t()` / `tf()`, the messages in 9 languages, and the ones a
//! project adds in its own `locales/` folder (`load_project`).
//!
//! Used through `runique::utils::translation`, which re-exports this crate.
pub mod keys;
pub mod project;
pub mod switch_lang;

pub use project::{LanguageReport, LocaleError, ProjectReport, load_project, project_dir};
pub use switch_lang::{Lang, LangId, current_lang, set_default_lang, set_lang, with_lang};
pub use switch_lang::{t, tf};
