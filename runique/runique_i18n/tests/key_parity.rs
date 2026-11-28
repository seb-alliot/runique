//! Every translation file must hold exactly the keys of `en.json`, each with
//! as many `{}` placeholders. `Lang::get` quietly falls back to English, then
//! to the raw key, so a missing key never shows up anywhere else; and a
//! placeholder too few drops an argument without a word. A `\\n` written
//! in the JSON also gets caught: it prints as a literal backslash-n instead
//! of a line break.

use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

const LANGS: &[&str] = &["fr", "it", "es", "de", "pt", "ja", "zh", "ru"];

fn locales() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/locales")
}

/// The files of a language folder, by their path in it (`forms.json`,
/// `admin/login.json`), sorted.
fn theme_files(code: &str) -> Vec<String> {
    fn walk(root: &Path, dir: &Path, found: &mut Vec<String>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(root, &path, found);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                found.push(relative.replace('\\', "/"));
            }
        }
    }
    let folder = locales().join(code);
    let mut names = Vec::new();
    walk(&folder, &folder, &mut names);
    // The language's own description: not a subject, it has no key.
    names.retain(|name| name != "lang.json");
    names.sort();
    names
}

/// A whole language as one tree: each file under its path, a folder being
/// one more level (`admin/login.json` is `admin.login`). The keys are then
/// the ones the code writes.
fn language(code: &str) -> Value {
    let mut tree = serde_json::Map::new();
    for name in theme_files(code) {
        let path = locales().join(code).join(&name);
        let dotted = name.strip_suffix(".json").unwrap_or_else(|| {
            panic!(
                "{}: only .json files belong in a language folder",
                path.display()
            )
        });
        let raw = fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} isn't valid JSON: {e}", path.display()));
        let mut parts: Vec<&str> = dotted.split('/').collect();
        let file = parts.pop().unwrap();
        let mut node = &mut tree;
        for folder in parts {
            node = node
                .entry(folder.to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()))
                .as_object_mut()
                .unwrap();
        }
        node.insert(file.to_string(), json);
    }
    Value::Object(tree)
}

/// Each dotted key with its number of placeholders.
fn flatten(value: &Value, prefix: &str, out: &mut BTreeMap<String, usize>) {
    flatten_with(value, prefix, &mut |key, text| {
        out.insert(key, text.matches("{}").count());
    });
}

fn flatten_with(value: &Value, prefix: &str, visit: &mut impl FnMut(String, &str)) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                flatten_with(child, &format!("{prefix}{key}."), visit);
            }
        }
        Value::String(text) => visit(prefix.trim_end_matches('.').to_string(), text),
        _ => {}
    }
}

fn keys_of(code: &str) -> BTreeMap<String, usize> {
    let mut keys = BTreeMap::new();
    flatten(&language(code), "", &mut keys);
    keys
}

// The nine folders mirror each other: the same theme files in each.
#[test]
fn every_language_folder_has_the_english_theme_files() {
    let english = theme_files("en");
    assert!(!english.is_empty());
    for code in LANGS {
        assert_eq!(theme_files(code), english, "{code}");
    }
    let mut folders: Vec<String> = fs::read_dir(locales())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    folders.sort();
    let mut expected: Vec<String> = LANGS.iter().map(|code| code.to_string()).collect();
    expected.push("en".to_string());
    expected.sort();
    assert_eq!(
        folders, expected,
        "a folder that isn't a language of Runique"
    );
}

#[test]
fn every_language_has_the_english_keys_and_placeholders() {
    let english = keys_of("en");
    let mut problems = Vec::new();

    for code in LANGS {
        let keys = keys_of(code);
        for (key, count) in &english {
            match keys.get(key) {
                None => problems.push(format!("{code}: missing {key}")),
                Some(n) if n != count => problems.push(format!(
                    "{code}: {key} has {n} placeholder(s), English has {count}"
                )),
                Some(_) => {}
            }
        }
        for key in keys.keys().filter(|key| !english.contains_key(*key)) {
            problems.push(format!("{code}: {key} isn't in English"));
        }
    }

    assert!(
        problems.is_empty(),
        "{} translation problem(s):\n{}",
        problems.len(),
        problems.join("\n")
    );
}

#[test]
fn no_translation_holds_a_literal_backslash_n() {
    let mut problems = Vec::new();
    for code in LANGS.iter().chain(["en"].iter()) {
        flatten_with(&language(code), "", &mut |key, text| {
            if text.contains("\\n") {
                problems.push(format!("{code}: {key}"));
            }
        });
    }
    assert!(
        problems.is_empty(),
        "literal \\n instead of a line break in:\n{}",
        problems.join("\n")
    );
}

// The constants of `keys` and the English files hold each other: a key
// without its message would show as the key itself, a message without its
// constant can't be asked for by name.
#[test]
fn every_key_constant_has_its_message_and_every_message_its_constant() {
    let mut english = BTreeMap::new();
    flatten(&language("en"), "", &mut english);

    let constants: std::collections::BTreeSet<&str> =
        runique_i18n::keys::ALL.iter().copied().collect();
    assert_eq!(
        constants.len(),
        runique_i18n::keys::ALL.len(),
        "a key twice"
    );

    let without_message: Vec<&&str> = constants
        .iter()
        .filter(|key| !english.contains_key(**key))
        .collect();
    let without_constant: Vec<&String> = english
        .keys()
        .filter(|key| !constants.contains(key.as_str()))
        .collect();
    assert!(
        without_message.is_empty(),
        "no message for: {without_message:?}"
    );
    assert!(
        without_constant.is_empty(),
        "no constant for: {without_constant:?}"
    );
}

// A message is found under its constant, in every language, without the
// fallback to English coming into play.
#[test]
fn a_constant_finds_its_message_in_every_language() {
    use runique_i18n::Lang;
    let key = runique_i18n::keys::forms::error::REQUIRED;
    let english = Lang::En.get(key);
    assert_ne!(english, key, "the key itself means: not found");
    assert_ne!(Lang::Fr.get(key), english);
    assert_ne!(Lang::Fr.get(key), key);
    // In a folder: the screen is part of the file's name.
    let nested = runique_i18n::keys::admin::login::label::TITLE;
    assert_ne!(Lang::En.get(nested), nested);
    assert_ne!(Lang::Fr.get(nested), nested);
    // Half a key, or a key cut in its file's name, finds nothing.
    for wrong in [
        "forms",
        "forms.",
        "forms.error",
        "admin.log",
        "admin.login",
        "form.error.required",
    ] {
        assert_eq!(Lang::En.get(wrong), wrong, "{wrong}");
    }
}
