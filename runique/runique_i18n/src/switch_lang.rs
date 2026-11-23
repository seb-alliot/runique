//! The language in use, and the lookup of a message by its key.

use serde_json::Value;
use std::{
    borrow::Cow,
    fmt::Display,
    sync::{
        LazyLock,
        atomic::{AtomicU8, Ordering},
    },
};

/// Languages supported by Runique
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Fr,
    #[default]
    En,
    It,
    Es,
    De,
    Pt,
    Ja,
    Zh,
    Ru,
}

static GLOBAL_LANG: AtomicU8 = AtomicU8::new(1); // 1 = Lang::En

/// The messages on disk: one folder per language, one file per theme
/// (`locales/<language>/<theme>.json`), the nine folders mirroring each other.
/// A key is `theme.key`: its first part names the file.
///
/// Declared once here: a new theme is one more name in the list, and its file
/// in every language folder (the build fails on a missing one).
macro_rules! locales {
    (languages: [$($lang:ident => $code:literal),* $(,)?], themes: $themes:tt $(,)?) => {
        const THEMES: &[&str] = locales!(@names $themes);

        /// The files of `lang`, in the order of [`THEMES`].
        fn sources(lang: Lang) -> &'static [&'static str] {
            match lang {
                $(Lang::$lang => locales!(@files $code, $themes),)*
            }
        }
    };
    (@names [$($theme:literal),* $(,)?]) => {
        &[$($theme),*]
    };
    (@files $code:literal, [$($theme:literal),* $(,)?]) => {
        &[$(include_str!(concat!("locales/", $code, "/", $theme, ".json"))),*]
    };
}

locales! {
    languages: [
        Fr => "fr", En => "en", It => "it", Es => "es", De => "de",
        Pt => "pt", Ja => "ja", Zh => "zh", Ru => "ru",
    ],
    themes: [
        "admin", "atomic", "auth", "build", "cli", "csrf", "database", "debug",
        "error", "flash", "forms", "html", "log", "makemigrations", "middleware",
        "migrate", "reset", "runique_test", "server", "TemplateMessage",
        "test_runner", "title",
    ],
}

/// The themes of one language, parsed, in the order of [`THEMES`].
type Catalog = Vec<Value>;

fn catalog(lang: Lang) -> Catalog {
    sources(lang)
        .iter()
        .zip(THEMES)
        .map(|(text, theme)| {
            serde_json::from_str(text).unwrap_or_else(|e| {
                panic!(
                    "Invalid translation file '{}/{theme}.json': {e}",
                    lang.code()
                )
            })
        })
        .collect()
}

/// Each language is parsed the first time a message is asked in it. In the
/// order of [`Lang::as_u8`].
static CATALOGS: [LazyLock<Catalog>; 9] = [
    LazyLock::new(|| catalog(Lang::Fr)),
    LazyLock::new(|| catalog(Lang::En)),
    LazyLock::new(|| catalog(Lang::It)),
    LazyLock::new(|| catalog(Lang::Es)),
    LazyLock::new(|| catalog(Lang::De)),
    LazyLock::new(|| catalog(Lang::Pt)),
    LazyLock::new(|| catalog(Lang::Ja)),
    LazyLock::new(|| catalog(Lang::Zh)),
    LazyLock::new(|| catalog(Lang::Ru)),
];

/// Dynamically sets the global language of the application.
pub fn set_lang(lang: Lang) {
    GLOBAL_LANG.store(lang.as_u8(), Ordering::Relaxed);
}

/// Returns the configured global language (default: `En`).
pub fn current_lang() -> Lang {
    Lang::from_u8(GLOBAL_LANG.load(Ordering::Relaxed))
}

/// Translates a key with the global language.
#[must_use]
pub fn t(key: &str) -> Cow<'static, str> {
    current_lang().get(key)
}

/// Translates a key with parameters using the global language.
pub fn tf<T: Display>(key: &str, args: &[T]) -> String {
    current_lang().format(key, args)
}

impl From<&str> for Lang {
    fn from(s: &str) -> Self {
        // Normalize locale strings: "fr_FR.UTF-8" → "fr-fr", "fr" → "fr"
        let normalized = s
            .split('.')
            .next()
            .unwrap_or(s)
            .replace('_', "-")
            .to_lowercase();

        match normalized.as_str() {
            "fr" | "fr-fr" | "fr-ca" | "fr-be" | "fr-ch" => Lang::Fr,
            "it" | "it-it" | "it-ch" => Lang::It,
            "es" | "es-es" | "es-mx" | "es-ar" | "es-co" | "es-cl" => Lang::Es,
            "de" | "de-de" | "de-at" | "de-ch" => Lang::De,
            "pt" | "pt-pt" | "pt-br" => Lang::Pt,
            "ja" | "ja-jp" => Lang::Ja,
            "zh" | "zh-cn" | "zh-tw" | "zh-hk" => Lang::Zh,
            "ru" | "ru-ru" | "ru-by" | "ru-ua" => Lang::Ru,
            _ => Lang::En,
        }
    }
}

impl From<String> for Lang {
    fn from(s: String) -> Self {
        Lang::from(s.as_str())
    }
}

impl Lang {
    /// The language of the user's locale, read from `LANG`, then `LC_ALL`,
    /// then `LC_MESSAGES`. `None` when none of them is set.
    #[must_use]
    pub fn from_env() -> Option<Lang> {
        ["LANG", "LC_ALL", "LC_MESSAGES"]
            .iter()
            .find_map(|key| std::env::var(key).ok())
            .map(Lang::from)
    }

    /// The language the `.env` file sets, under the same keys, read without
    /// touching the environment: the system always sets `LANG`, so the file's
    /// would never win otherwise, while every other key (`DATABASE_URL`…)
    /// keeps the environment first. `None` without a `.env` or such a key.
    #[must_use]
    pub fn from_dotenv() -> Option<Lang> {
        let vars: Vec<(String, String)> = dotenvy::dotenv_iter()
            .ok()?
            .filter_map(Result::ok)
            .collect();
        ["LANG", "LC_ALL", "LC_MESSAGES"]
            .iter()
            .find_map(|key| vars.iter().find(|(k, _)| k == key))
            .map(|(_, v)| Lang::from(v.as_str()))
    }

    fn as_u8(self) -> u8 {
        match self {
            Lang::Fr => 0,
            Lang::En => 1,
            Lang::It => 2,
            Lang::Es => 3,
            Lang::De => 4,
            Lang::Pt => 5,
            Lang::Ja => 6,
            Lang::Zh => 7,
            Lang::Ru => 8,
        }
    }

    fn from_u8(v: u8) -> Self {
        match v {
            0 => Lang::Fr,
            1 => Lang::En,
            2 => Lang::It,
            3 => Lang::Es,
            4 => Lang::De,
            5 => Lang::Pt,
            6 => Lang::Ja,
            7 => Lang::Zh,
            8 => Lang::Ru,
            other => {
                tracing::warn!(
                    value = other,
                    "unknown language code, falling back to the default"
                );
                Lang::default()
            }
        }
    }

    /// Returns the language code (for file names)
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
            Lang::It => "it",
            Lang::Es => "es",
            Lang::De => "de",
            Lang::Pt => "pt",
            Lang::Ja => "ja",
            Lang::Zh => "zh",
            Lang::Ru => "ru",
        }
    }

    /// Retrieves a translated message by its key (e.g., "forms.required")
    #[must_use]
    pub fn get(&self, key: &str) -> Cow<'static, str> {
        if let Some(s) = self.lookup(key) {
            return s;
        }
        if *self != Lang::En
            && let Some(s) = Lang::En.lookup(key)
        {
            return s;
        }
        Cow::Owned(key.to_string())
    }

    /// The first part of the key names the theme, so the file; the rest is
    /// looked up inside it.
    fn lookup(self, key: &str) -> Option<Cow<'static, str>> {
        let (theme, rest) = key.split_once('.')?;
        let at = THEMES.iter().position(|name| *name == theme)?;
        let mut current = &CATALOGS[usize::from(self.as_u8())][at];
        for part in rest.split('.') {
            current = current.get(part)?;
        }
        current.as_str().map(Cow::Borrowed)
    }

    /// Retrieves a message with parameters (replaces {} with args)
    pub fn format<T: Display>(self, key: &str, args: &[T]) -> String {
        let template = self.get(key);
        let mut result = template.to_string();

        for arg in args {
            result = result.replacen("{}", &arg.to_string(), 1);
        }

        result
    }
}
