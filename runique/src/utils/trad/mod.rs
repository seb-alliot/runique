//! Internationalization — global language selection (`set_lang`), `t()` / `tf()` helpers.
//! The engine and the messages live in the `runique_i18n` crate.
#![doc = include_str!("../../../doc-tests/i18n/switch_lang.md")]
pub use runique_i18n::keys;
pub use runique_i18n::switch_lang;

pub use runique_i18n::project::{
    LanguageReport, LocaleError, ProjectReport, load_project, project_dir,
};
pub use switch_lang::{Lang, LangId, current_lang, set_default_lang, set_lang};
pub use switch_lang::{t, tf};
