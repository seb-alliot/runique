//! Test — complétude i18n de `ADMIN_MESSAGE_KEYS` sur les 9 langues.
//!
//! Deuxième test du chantier "détection des bugs muets" (cf. mémoire projet
//! `project_tests_detection_bugs_muets`). Motivation : `t()`/`Lang::get()`
//! retombe silencieusement sur l'anglais puis sur la clé brute si absente —
//! aucun test de rendu (route crawl inclus) ne peut voir une clé manquante
//! dans une langue tant qu'elle existe en anglais. Ce test lit les 9 fichiers
//! JSON directement (via `include_str!`, contournant le fallback de `Lang::get`)
//! et vérifie que chaque clé de `ADMIN_MESSAGE_KEYS` y résout réellement.

use runique::utils::constante::ADMIN_MESSAGE_KEYS;
use serde_json::Value;

const LANGS: &[(&str, &str)] = &[
    ("fr", include_str!("../../src/utils/trad/fr.json")),
    ("en", include_str!("../../src/utils/trad/en.json")),
    ("it", include_str!("../../src/utils/trad/it.json")),
    ("es", include_str!("../../src/utils/trad/es.json")),
    ("de", include_str!("../../src/utils/trad/de.json")),
    ("pt", include_str!("../../src/utils/trad/pt.json")),
    ("ja", include_str!("../../src/utils/trad/ja.json")),
    ("zh", include_str!("../../src/utils/trad/zh.json")),
    ("ru", include_str!("../../src/utils/trad/ru.json")),
];

/// Résout une clé pointée ("admin.login.title") dans un JSON imbriqué, sans
/// aucun fallback — contrairement à `Lang::get()`.
fn resolves_to_string(json: &Value, dotted_key: &str) -> bool {
    let mut current = json;
    for part in dotted_key.split('.') {
        match current.get(part) {
            Some(v) => current = v,
            None => return false,
        }
    }
    current.as_str().is_some()
}

#[test]
fn admin_message_keys_resolve_in_every_language() {
    let parsed: Vec<(&str, Value)> = LANGS
        .iter()
        .map(|(code, raw)| {
            (
                *code,
                serde_json::from_str(raw).unwrap_or_else(|e| panic!("{code}.json invalide : {e}")),
            )
        })
        .collect();

    let mut missing: Vec<String> = Vec::new();
    for key in ADMIN_MESSAGE_KEYS {
        for (code, json) in &parsed {
            if !resolves_to_string(json, key) {
                missing.push(format!("{code}: {key}"));
            }
        }
    }

    assert!(
        missing.is_empty(),
        "{} clé(s) de ADMIN_MESSAGE_KEYS absentes d'au moins une langue :\n{}",
        missing.len(),
        missing.join("\n")
    );
}
