//! Internationalization — the language in use, the `t()` / `tf()` helpers
//! and the project's own translations.
//! The engine and the messages live in the `runique_i18n` crate.
#![doc = include_str!("../../../doc-tests/i18n/switch_lang.md")]
pub use runique_i18n::keys;

pub use runique_i18n::project::{
    LanguageReport, LocaleError, ProjectReport, load_project, project_dir,
};
pub use runique_i18n::{Lang, LangId, current_lang, t, tf, with_lang};
