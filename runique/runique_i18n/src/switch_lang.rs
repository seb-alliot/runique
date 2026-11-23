//! The language in use, and the lookup of a message by its key.
//!
//! A language is a folder of `locales/`: the nine Runique ships, embedded in
//! this crate, and the ones a project brings in its own `locales/` folder
//! (see [`crate::project`]). A message is looked for in the project's files
//! first, then in Runique's, then in the language it falls back on.

use serde_json::Value;
use std::{
    borrow::Cow,
    fmt::Display,
    sync::{
        LazyLock, PoisonError, RwLock, RwLockReadGuard,
        atomic::{AtomicUsize, Ordering},
    },
};

/// A language: one of the nine Runique ships, or one found in the project's
/// `locales/` folder.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    /// A language of the project. Only loading its folder gives one: it is
    /// obtained with `Lang::from("nl")`, never written by hand.
    Other(LangId),
}

/// The place of a project's language among the ones loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LangId(u16);

/// Runique's own languages, in the order of the `languages` list below.
const BUILT_IN: [Lang; 9] = [
    Lang::Fr,
    Lang::En,
    Lang::It,
    Lang::Es,
    Lang::De,
    Lang::Pt,
    Lang::Ja,
    Lang::Zh,
    Lang::Ru,
];

/// English: where every chain of fallbacks ends.
pub(crate) const ENGLISH: usize = 1;

/// No language chosen yet: English until one is.
const UNSET: usize = usize::MAX;

static GLOBAL_LANG: AtomicUsize = AtomicUsize::new(UNSET);

/// The messages on disk: one folder per language, one file per subject
/// (`locales/<language>/<subject>.json`), the folders mirroring each other,
/// plus the language's own `lang.json` (its name, its regional variants, the
/// language it falls back on). A key is `subject.section.key`: its first part
/// names the file.
///
/// `admin` is a folder, one file per screen: its keys are
/// `admin.<screen>.<section>.<key>`.
///
/// Declared once here: a new subject is one more name in the list, and its
/// file in every language folder (the build fails on a missing one).
macro_rules! locales {
    (
        languages: [$($code:literal),* $(,)?],
        subjects: $subjects:tt,
        folders: $folders:tt $(,)?
    ) => {
        /// The folder of each language, in the order of [`BUILT_IN`].
        const CODES: &[&str] = &[$($code,)*];

        /// The `lang.json` of each language.
        const FICHES: &[&str] = &[$(include_str!(concat!("locales/", $code, "/lang.json")),)*];

        /// The dotted start of the keys of each file.
        const THEMES: &[&str] = locales!(@names $subjects $folders);

        /// The files of each language, in the order of [`THEMES`].
        const SOURCES: &[&[&str]] = &[$(locales!(@files $code, $subjects $folders),)*];
    };
    (
        @names
        [$($subject:literal),* $(,)?]
        [$($folder:literal => [$($file:literal),* $(,)?]),* $(,)?]
    ) => {
        &[
            $($subject,)*
            $($(concat!($folder, ".", $file),)*)*
        ]
    };
    (
        @files $code:literal,
        [$($subject:literal),* $(,)?]
        [$($folder:literal => [$($file:literal),* $(,)?]),* $(,)?]
    ) => {
        &[
            $(include_str!(concat!("locales/", $code, "/", $subject, ".json")),)*
            $($(include_str!(concat!("locales/", $code, "/", $folder, "/", $file, ".json")),)*)*
        ]
    };
}

locales! {
    languages: ["fr", "en", "it", "es", "de", "pt", "ja", "zh", "ru"],
    subjects: [
        "app", "atomic", "auth", "build", "cli", "csrf", "database",
        "debug", "forms", "http", "mailer", "makemigrations",
        "middleware", "migrate", "reset", "runique_test", "server", "session",
        "superuser", "test_runner",
    ],
    folders: [
        "admin" => [
            "access", "base", "bulk", "common", "create", "dashboard",
            "delete", "detail", "edit", "history", "list", "log",
            "login", "logout", "rank", "reset_password", "rights", "transfer",
            "user_created",
        ],
    ],
}

/// Puts `json` in `tree` under `path`: the file `admin/login.json` goes under
/// `admin`, then `login`.
pub(crate) fn insert_at(tree: &mut serde_json::Map<String, Value>, path: &[&str], json: Value) {
    let Some((last, folders)) = path.split_last() else {
        return;
    };
    let mut node = tree;
    for folder in folders {
        node = match node
            .entry((*folder).to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()))
        {
            Value::Object(map) => map,
            // A file and a folder of the same name: the file was there first.
            _ => return,
        };
    }
    node.insert((*last).to_string(), json);
}

/// The message under `key` in `tree`, one level per part of the key.
fn walk(tree: &'static Value, key: &str) -> Option<&'static str> {
    let mut current = tree;
    for part in key.split('.') {
        current = current.get(part)?;
    }
    current.as_str()
}

/// Every message of one of Runique's languages, as one tree.
fn base_tree(at: usize) -> Value {
    let mut tree = serde_json::Map::new();
    for (text, theme) in SOURCES[at].iter().zip(THEMES) {
        let json = serde_json::from_str(text).unwrap_or_else(|e| {
            panic!("Invalid translation file '{}/{theme}.json': {e}", CODES[at])
        });
        let path: Vec<&str> = theme.split('.').collect();
        insert_at(&mut tree, &path, json);
    }
    Value::Object(tree)
}

/// Each language is parsed the first time a message is asked in it. In the
/// order of [`BUILT_IN`].
static BASE: [LazyLock<Value>; 9] = [
    LazyLock::new(|| base_tree(0)),
    LazyLock::new(|| base_tree(1)),
    LazyLock::new(|| base_tree(2)),
    LazyLock::new(|| base_tree(3)),
    LazyLock::new(|| base_tree(4)),
    LazyLock::new(|| base_tree(5)),
    LazyLock::new(|| base_tree(6)),
    LazyLock::new(|| base_tree(7)),
    LazyLock::new(|| base_tree(8)),
];

/// Runique's English messages: what a project's files are checked against.
pub(crate) fn english() -> &'static Value {
    &BASE[ENGLISH]
}

/// What a `lang.json` says of its language.
pub(crate) struct Fiche {
    pub(crate) name: Option<String>,
    /// Other ways to write the language (`fr-ca`), lowercase.
    pub(crate) aliases: Vec<String>,
    /// The language to look in when a message is missing; English when unsaid.
    pub(crate) fallback: Option<String>,
}

impl Fiche {
    /// Reads a `lang.json`. A field of the wrong type is an error, never a
    /// value quietly dropped.
    pub(crate) fn parse(text: &str) -> Result<Self, String> {
        let json: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
        let Some(fields) = json.as_object() else {
            return Err("expected an object".to_string());
        };
        let text_of = |field: &str| match fields.get(field) {
            None => Ok(None),
            Some(Value::String(value)) => Ok(Some(value.clone())),
            Some(_) => Err(format!("`{field}` must be a string")),
        };
        let aliases = match fields.get("aliases") {
            None => Vec::new(),
            Some(Value::Array(values)) => values
                .iter()
                .map(|value| {
                    value
                        .as_str()
                        .map(normalize)
                        .ok_or_else(|| "`aliases` must be a list of strings".to_string())
                })
                .collect::<Result<_, _>>()?,
            Some(_) => return Err("`aliases` must be a list of strings".to_string()),
        };
        Ok(Self {
            name: text_of("name")?,
            aliases,
            fallback: text_of("fallback")?.map(|code| normalize(&code)),
        })
    }
}

/// A language as loaded: what it is called, and where its messages are.
pub(crate) struct Language {
    pub(crate) code: &'static str,
    pub(crate) name: &'static str,
    pub(crate) aliases: Vec<String>,
    /// The place of the language to look in next. English points to itself.
    pub(crate) fallback: usize,
    /// The project's files for this language: looked in first.
    pub(crate) project: Option<&'static Value>,
    /// Runique's files for this language, when it is one of its own.
    base: Option<&'static LazyLock<Value>>,
}

impl Language {
    /// A language Runique doesn't ship: its messages are the project's.
    pub(crate) fn of_project(code: &'static str, name: &'static str, aliases: Vec<String>) -> Self {
        Self {
            code,
            name,
            aliases,
            fallback: ENGLISH,
            project: None,
            base: None,
        }
    }

    /// The message of `key` in this language's own files, the project's first.
    pub(crate) fn find(&self, key: &str) -> Option<&'static str> {
        self.project
            .and_then(|tree| walk(tree, key))
            .or_else(|| self.base.and_then(|tree| walk(tree, key)))
    }
}

/// Every language loaded: Runique's nine first, then the project's.
pub(crate) struct Registry {
    pub(crate) languages: Vec<Language>,
}

impl Registry {
    /// Runique's languages alone, as their `lang.json` describe them.
    pub(crate) fn built_in() -> Self {
        let languages = CODES
            .iter()
            .zip(FICHES)
            .zip(&BASE)
            .map(|((&code, fiche), base)| {
                let fiche = Fiche::parse(fiche)
                    .unwrap_or_else(|e| panic!("Invalid 'locales/{code}/lang.json': {e}"));
                let fallback = fiche
                    .fallback
                    .and_then(|wanted| CODES.iter().position(|code| *code == wanted))
                    .unwrap_or(ENGLISH);
                Language {
                    code,
                    name: fiche.name.map_or(code, |name| &*name.leak()),
                    aliases: fiche.aliases,
                    fallback,
                    project: None,
                    base: Some(base),
                }
            })
            .collect();
        Self { languages }
    }

    /// The place of the language written `name`: its code, or one of its
    /// aliases. `name` is expected normalized.
    pub(crate) fn position(&self, name: &str) -> Option<usize> {
        self.languages.iter().position(|language| {
            language.code == name || language.aliases.iter().any(|a| a == name)
        })
    }

    /// The message of `key` for the language at `at`, following its
    /// fallbacks. A chain is never longer than the number of languages:
    /// loading refuses a loop.
    fn message(&self, at: usize, key: &str) -> Option<&'static str> {
        let mut at = if at < self.languages.len() {
            at
        } else {
            ENGLISH
        };
        for _ in &self.languages {
            let language = self.languages.get(at)?;
            if let Some(text) = language.find(key) {
                return Some(text);
            }
            if language.fallback == at {
                return None;
            }
            at = language.fallback;
        }
        None
    }
}

pub(crate) static REGISTRY: LazyLock<RwLock<Registry>> =
    LazyLock::new(|| RwLock::new(Registry::built_in()));

/// The languages as they are now. A thread that panicked while loading
/// leaves the former ones in place: they are still read.
fn registry() -> RwLockReadGuard<'static, Registry> {
    REGISTRY.read().unwrap_or_else(PoisonError::into_inner)
}

/// A language name as it is compared: `fr_FR.UTF-8` and `FR-fr` are `fr-fr`.
pub(crate) fn normalize(name: &str) -> String {
    name.split('.')
        .next()
        .unwrap_or(name)
        .trim()
        .replace('_', "-")
        .to_lowercase()
}

/// Sets the language of the application: the one asked for comes before any
/// default (see [`set_default_lang`]).
pub fn set_lang(lang: Lang) {
    GLOBAL_LANG.store(lang.index(), Ordering::Relaxed);
}

/// Sets the language unless one was already chosen with [`set_lang`]: for a
/// default read from the environment, which must not undo a choice made in
/// the code.
pub fn set_default_lang(lang: Lang) {
    let _ = GLOBAL_LANG.compare_exchange(UNSET, lang.index(), Ordering::Relaxed, Ordering::Relaxed);
}

/// Returns the language in use (default: `En`). A language of the project
/// that is no longer loaded counts as English.
pub fn current_lang() -> Lang {
    let at = GLOBAL_LANG.load(Ordering::Relaxed);
    if at < registry().languages.len() {
        Lang::from_index(at)
    } else {
        Lang::En
    }
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
    /// The language written `s`: its code or one of the aliases of its
    /// `lang.json` (`fr`, `fr-CA`, `fr_FR.UTF-8`). English for anything else.
    fn from(s: &str) -> Self {
        registry()
            .position(&normalize(s))
            .map_or(Lang::En, Lang::from_index)
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

    /// Every language loaded: Runique's nine, then the project's.
    #[must_use]
    pub fn available() -> Vec<Lang> {
        (0..registry().languages.len())
            .map(Lang::from_index)
            .collect()
    }

    /// The place of the language among the ones loaded.
    fn index(self) -> usize {
        match self {
            Lang::Other(LangId(n)) => BUILT_IN.len().saturating_add(usize::from(n)),
            built_in => BUILT_IN
                .iter()
                .position(|lang| *lang == built_in)
                .unwrap_or(ENGLISH),
        }
    }

    pub(crate) fn from_index(at: usize) -> Self {
        match BUILT_IN.get(at) {
            Some(lang) => *lang,
            None => at
                .checked_sub(BUILT_IN.len())
                .and_then(|n| u16::try_from(n).ok())
                .map_or(Lang::En, |n| Lang::Other(LangId(n))),
        }
    }

    /// Returns the language code: the name of its folder (`fr`, `nl`).
    #[must_use]
    pub fn code(&self) -> &'static str {
        registry()
            .languages
            .get(self.index())
            .map_or(CODES[ENGLISH], |language| language.code)
    }

    /// The name of the language, in that language (`Français`): the `name`
    /// of its `lang.json`, or its code when it gives none.
    #[must_use]
    pub fn name(&self) -> &'static str {
        registry()
            .languages
            .get(self.index())
            .map_or(CODES[ENGLISH], |language| language.name)
    }

    /// Retrieves a translated message by its key (e.g., `keys::forms::error::REQUIRED`):
    /// in the project's files, then Runique's, then the language it falls
    /// back on, down to English. The key itself when nothing has it.
    #[must_use]
    pub fn get(&self, key: &str) -> Cow<'static, str> {
        registry()
            .message(self.index(), key)
            .map_or_else(|| Cow::Owned(key.to_string()), Cow::Borrowed)
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
