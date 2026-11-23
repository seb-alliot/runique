//! Every translation file must hold exactly the keys of `en.json`, each with
//! as many `{}` placeholders. `Lang::get` quietly falls back to English, then
//! to the raw key, so a missing key never shows up anywhere else; and a
//! placeholder too few drops an argument without a word. A `\\n` written
//! in the JSON also gets caught: it prints as a literal backslash-n instead
//! of a line break.

use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const LANGS: &[&str] = &["fr", "it", "es", "de", "pt", "ja", "zh", "ru"];

fn locales() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/locales")
}

/// The theme files of a language folder, by name, sorted.
fn theme_files(code: &str) -> Vec<String> {
    let folder = locales().join(code);
    let mut names: Vec<String> = fs::read_dir(&folder)
        .unwrap_or_else(|e| panic!("{}: {e}", folder.display()))
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// A whole language as one tree, `{ theme: its file }`: the keys are then the
/// ones the code writes (`theme.key`).
fn language(code: &str) -> Value {
    let mut themes = serde_json::Map::new();
    for name in theme_files(code) {
        let path = locales().join(code).join(&name);
        let theme = name.strip_suffix(".json").unwrap_or_else(|| {
            panic!("{}: only .json files belong in a language folder", path.display())
        });
        let raw = fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} isn't valid JSON: {e}", path.display()));
        themes.insert(theme.to_string(), json);
    }
    Value::Object(themes)
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
    assert_eq!(folders, expected, "a folder that isn't a language of Runique");
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
