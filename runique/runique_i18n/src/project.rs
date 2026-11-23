//! The translations a project brings: its own `locales/` folder, laid out as
//! Runique's (`locales/<language>/<subject>.json`).
//!
//! A folder named after one of Runique's languages adds to it: a message of
//! the same key takes the place of Runique's, a subject of the project's own
//! (`shop.json`) gives it its keys (`shop.title`). A folder of another name
//! is a language Runique doesn't ship, described by its `lang.json`.
//!
//! The folder is read once, when the application or the command starts.
use crate::switch_lang::{
    ENGLISH, Fiche, Language, REGISTRY, Registry, english, insert_at, normalize,
};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
    sync::PoisonError,
};

/// The folder read when `LOCALES_DIR` doesn't name another.
pub const DEFAULT_DIR: &str = "locales";

/// The file of a language folder that describes the language itself.
const FICHE: &str = "lang.json";

/// The project's folder: `LOCALES_DIR`, or `locales` under the current folder.
#[must_use]
pub fn project_dir() -> PathBuf {
    std::env::var_os("LOCALES_DIR")
        .filter(|dir| !dir.is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_DIR), PathBuf::from)
}

/// Why a project's folder was refused. Nothing of it is kept: the languages
/// stay as they were.
#[derive(Debug, PartialEq, Eq)]
pub enum LocaleError {
    /// A folder or a file that can't be read.
    Unreadable { path: PathBuf, reason: String },
    /// A folder whose name isn't a language code.
    NotALanguage { path: PathBuf },
    /// A file that isn't valid JSON, or a `lang.json` with a wrong field.
    Invalid { path: PathBuf, reason: String },
    /// A value that is neither a text nor a section.
    NotText { path: PathBuf, key: String },
    /// Two languages under the same code or alias.
    SameName { name: String },
    /// A `fallback` that names no language.
    UnknownFallback { language: String, fallback: String },
    /// Fallbacks that never reach English.
    FallbackLoop { language: String },
    /// More languages than can be numbered.
    TooMany,
}

impl fmt::Display for LocaleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::NotALanguage { path } => write!(
                f,
                "{}: a language folder is named by its code, in lowercase (fr, pt-br)",
                path.display()
            ),
            Self::Invalid { path, reason } => write!(f, "{}: {reason}", path.display()),
            Self::NotText { path, key } => write!(
                f,
                "{}: `{key}` is neither a text nor a section",
                path.display()
            ),
            Self::SameName { name } => write!(f, "`{name}` names two languages"),
            Self::UnknownFallback { language, fallback } => write!(
                f,
                "{language}/{FICHE}: the fallback `{fallback}` is not a language"
            ),
            Self::FallbackLoop { language } => {
                write!(f, "{language}/{FICHE}: its fallbacks never reach English")
            }
            Self::TooMany => write!(f, "too many languages"),
        }
    }
}

impl std::error::Error for LocaleError {}

/// What was found for one language of the project's folder.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct LanguageReport {
    pub code: String,
    /// A language Runique doesn't ship.
    pub new: bool,
    /// How many messages the folder holds.
    pub messages: usize,
    /// For a new language: Runique's messages it doesn't translate, counted
    /// by subject. They are shown in the language it falls back on.
    pub missing: Vec<(String, usize)>,
    /// Keys written under one of Runique's subjects that Runique doesn't
    /// have: nothing asks for them, almost always a misspelt key.
    pub unknown: Vec<String>,
    /// Messages replacing one of Runique's with another number of `{}`:
    /// a parameter would be dropped, or left unfilled.
    pub mismatched: Vec<String>,
}

/// What a project's folder held, language by language, in the order of their
/// codes. Empty when the project has no folder.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ProjectReport {
    pub languages: Vec<LanguageReport>,
}

/// Reads the project's folder and puts its languages in place of the ones of
/// a former call. No folder at `dir`: Runique's languages alone.
///
/// On an error nothing changes: the languages are replaced only once the
/// whole folder has been read and checked.
pub fn load_project(dir: &Path) -> Result<ProjectReport, LocaleError> {
    let mut registry = Registry::built_in();
    let mut report = ProjectReport::default();
    let mut fallbacks: Vec<(usize, String)> = Vec::new();

    for (code, folder) in language_folders(dir)? {
        let (tree, fiche) = read_language(&folder)?;
        let messages = flatten(&tree);
        let known = registry.position(&code);
        let at = match known {
            Some(at) => at,
            None => {
                let fiche = fiche.unwrap_or(Fiche {
                    name: None,
                    aliases: Vec::new(),
                    fallback: None,
                });
                // Numbered in a `u16`: see `Lang::Other`.
                if registry.languages.len() >= usize::from(u16::MAX) {
                    return Err(LocaleError::TooMany);
                }
                let code: &'static str = code.clone().leak();
                let name: &'static str = fiche.name.map_or(code, |name| &*name.leak());
                fallbacks.push((
                    registry.languages.len(),
                    fiche.fallback.unwrap_or_else(|| "en".to_string()),
                ));
                registry
                    .languages
                    .push(Language::of_project(code, name, fiche.aliases));
                registry.languages.len().saturating_sub(1)
            }
        };
        report
            .languages
            .push(check(&code, known.is_none(), &messages));
        let tree: &'static Value = Box::leak(Box::new(Value::Object(tree)));
        registry.languages[at].project = Some(tree);
    }

    unique_names(&registry)?;
    for (at, wanted) in fallbacks {
        let Some(fallback) = registry.position(&wanted) else {
            return Err(LocaleError::UnknownFallback {
                language: registry.languages[at].code.to_string(),
                fallback: wanted,
            });
        };
        registry.languages[at].fallback = fallback;
    }
    reaches_english(&registry)?;

    *REGISTRY.write().unwrap_or_else(PoisonError::into_inner) = registry;
    Ok(report)
}

/// The language folders of `dir`, by code, sorted. Files at that level are
/// not languages: left alone.
fn language_folders(dir: &Path) -> Result<Vec<(String, PathBuf)>, LocaleError> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut folders = Vec::new();
    for path in entries(dir)? {
        if !path.is_dir() {
            continue;
        }
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !is_language_code(&name) {
            return Err(LocaleError::NotALanguage { path });
        }
        folders.push((name, path));
    }
    Ok(folders)
}

/// The entries of a folder, sorted: the same order on every machine.
fn entries(dir: &Path) -> Result<Vec<PathBuf>, LocaleError> {
    let unreadable = |e: std::io::Error| LocaleError::Unreadable {
        path: dir.to_path_buf(),
        reason: e.to_string(),
    };
    let mut paths = std::fs::read_dir(dir)
        .map_err(unreadable)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(unreadable)?;
    paths.sort();
    Ok(paths)
}

/// A code as a folder is named: lowercase letters and digits in parts joined
/// by `-`, a letter first (`fr`, `pt-br`, `zh-hant`). Never a path.
fn is_language_code(name: &str) -> bool {
    (2..=16).contains(&name.len())
        && name.starts_with(|c: char| c.is_ascii_lowercase())
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// The messages of a language folder as one tree, and its `lang.json` if it
/// has one. A folder inside it is one more level of the keys.
fn read_language(folder: &Path) -> Result<(Map<String, Value>, Option<Fiche>), LocaleError> {
    let mut tree = Map::new();
    let mut fiche = None;
    for (parts, path) in json_files(folder, folder)? {
        let text = std::fs::read_to_string(&path).map_err(|e| LocaleError::Unreadable {
            path: path.clone(),
            reason: e.to_string(),
        })?;
        let invalid = |reason: String| LocaleError::Invalid {
            path: path.clone(),
            reason,
        };
        if parts == ["lang"] {
            fiche = Some(Fiche::parse(&text).map_err(invalid)?);
            continue;
        }
        let json: Value = serde_json::from_str(&text).map_err(|e| invalid(e.to_string()))?;
        if !json.is_object() {
            return Err(invalid("expected an object".to_string()));
        }
        only_text(&json, &mut Vec::new(), &path)?;
        let parts: Vec<&str> = parts.iter().map(String::as_str).collect();
        insert_at(&mut tree, &parts, json);
    }
    Ok((tree, fiche))
}

/// Every `.json` file under `dir`, with its path from `root` as the parts of
/// a key (`admin/login.json` is `admin`, `login`). Other files are left alone.
fn json_files(root: &Path, dir: &Path) -> Result<Vec<(Vec<String>, PathBuf)>, LocaleError> {
    let mut files = Vec::new();
    for path in entries(dir)? {
        if path.is_dir() {
            files.extend(json_files(root, &path)?);
        } else if path.extension().is_some_and(|ext| ext == "json")
            && let Ok(relative) = path.with_extension("").strip_prefix(root)
        {
            let parts = relative
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect();
            files.push((parts, path));
        }
    }
    Ok(files)
}

/// Refuses a number, a list, a boolean or a `null` where a message is
/// expected: it would be asked for and never found.
fn only_text(json: &Value, key: &mut Vec<String>, path: &Path) -> Result<(), LocaleError> {
    match json {
        Value::String(_) => Ok(()),
        Value::Object(fields) => {
            for (name, value) in fields {
                key.push(name.clone());
                only_text(value, key, path)?;
                key.pop();
            }
            Ok(())
        }
        _ => Err(LocaleError::NotText {
            path: path.to_path_buf(),
            key: key.join("."),
        }),
    }
}

/// Each message of a tree under its dotted key.
fn flatten(tree: &Map<String, Value>) -> BTreeMap<String, &str> {
    fn walk<'a>(fields: &'a Map<String, Value>, prefix: &str, out: &mut BTreeMap<String, &'a str>) {
        for (name, value) in fields {
            let key = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{prefix}.{name}")
            };
            match value {
                Value::String(text) => {
                    out.insert(key, text);
                }
                Value::Object(inner) => walk(inner, &key, out),
                _ => {}
            }
        }
    }
    let mut out = BTreeMap::new();
    walk(tree, "", &mut out);
    out
}

/// Holds the messages of a project's language against Runique's English.
fn check(code: &str, new: bool, messages: &BTreeMap<String, &str>) -> LanguageReport {
    let empty = Map::new();
    let reference = flatten(english().as_object().unwrap_or(&empty));
    let subject = |key: &str| key.split('.').next().unwrap_or_default().to_string();
    let subjects: BTreeSet<String> = reference.keys().map(|key| subject(key)).collect();

    let mut report = LanguageReport {
        code: code.to_string(),
        new,
        messages: messages.len(),
        ..LanguageReport::default()
    };
    for (key, text) in messages {
        match reference.get(key) {
            Some(expected) => {
                if expected.matches("{}").count() != text.matches("{}").count() {
                    report.mismatched.push(key.clone());
                }
            }
            // A subject of the project's own is its business.
            None if subjects.contains(&subject(key)) => report.unknown.push(key.clone()),
            None => {}
        }
    }
    if new {
        let mut missing: BTreeMap<String, usize> = BTreeMap::new();
        for key in reference.keys().filter(|key| !messages.contains_key(*key)) {
            let count = missing.entry(subject(key)).or_default();
            *count = count.saturating_add(1);
        }
        report.missing = missing.into_iter().collect();
    }
    report
}

/// A code or an alias names one language only: `Lang::from` would otherwise
/// answer with the first one, whichever was meant.
fn unique_names(registry: &Registry) -> Result<(), LocaleError> {
    let mut seen = BTreeSet::new();
    for language in &registry.languages {
        let names =
            std::iter::once(normalize(language.code)).chain(language.aliases.iter().cloned());
        for name in names {
            if !seen.insert(name.clone()) {
                return Err(LocaleError::SameName { name });
            }
        }
    }
    Ok(())
}

/// Every language falls back, step by step, on English: a chain that comes
/// back on itself would never end.
fn reaches_english(registry: &Registry) -> Result<(), LocaleError> {
    for language in &registry.languages {
        let mut at = language.fallback;
        let mut steps = registry.languages.iter();
        while at != ENGLISH {
            let next = registry.languages.get(at).map(|language| language.fallback);
            match (steps.next(), next) {
                (Some(_), Some(next)) => at = next,
                _ => {
                    return Err(LocaleError::FallbackLoop {
                        language: language.code.to_string(),
                    });
                }
            }
        }
    }
    Ok(())
}
