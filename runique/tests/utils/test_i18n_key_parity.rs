//! Every translation file must hold exactly the keys of `en.json`, each with
//! as many `{}` placeholders. `Lang::get` quietly falls back to English, then
//! to the raw key, so a missing key never shows up anywhere else; and a
//! placeholder too few drops an argument without a word. A `\\n` written
//! in the JSON also gets caught: it prints as a literal backslash-n instead
//! of a line break.

use serde_json::Value;
use std::collections::BTreeMap;

const LANGS: &[(&str, &str)] = &[
    ("fr", include_str!("../../src/utils/trad/fr.json")),
    ("it", include_str!("../../src/utils/trad/it.json")),
    ("es", include_str!("../../src/utils/trad/es.json")),
    ("de", include_str!("../../src/utils/trad/de.json")),
    ("pt", include_str!("../../src/utils/trad/pt.json")),
    ("ja", include_str!("../../src/utils/trad/ja.json")),
    ("zh", include_str!("../../src/utils/trad/zh.json")),
    ("ru", include_str!("../../src/utils/trad/ru.json")),
];
const EN: &str = include_str!("../../src/utils/trad/en.json");

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

fn keys_of(code: &str, raw: &str) -> BTreeMap<String, usize> {
    let json: Value =
        serde_json::from_str(raw).unwrap_or_else(|e| panic!("{code}.json isn't valid JSON: {e}"));
    let mut keys = BTreeMap::new();
    flatten(&json, "", &mut keys);
    keys
}

#[test]
fn every_language_has_the_english_keys_and_placeholders() {
    let english = keys_of("en", EN);
    let mut problems = Vec::new();

    for (code, raw) in LANGS {
        let keys = keys_of(code, raw);
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
            problems.push(format!("{code}: {key} isn't in en.json"));
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
    for (code, raw) in LANGS.iter().chain([("en", EN)].iter()) {
        let json: Value = serde_json::from_str(raw)
            .unwrap_or_else(|e| panic!("{code}.json isn't valid JSON: {e}"));
        flatten_with(&json, "", &mut |key, text| {
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
