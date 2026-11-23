//! Test — complétude i18n de `ADMIN_MESSAGE_KEYS` sur les 9 langues.
//!
//! Deuxième test du chantier "détection des bugs muets" (cf. mémoire projet
//! `project_tests_detection_bugs_muets`). Motivation : `t()`/`Lang::get()`
//! retombe silencieusement sur l'anglais puis sur la clé brute si absente —
//! aucun test de rendu (route crawl inclus) ne peut voir une clé manquante
//! dans une langue tant qu'elle existe en anglais. Ce test lit le fichier `admin.json`
//! de chacune des 9 langues directement (via `include_str!`, contournant le fallback de `Lang::get`)
//! et vérifie que chaque clé de `ADMIN_MESSAGE_KEYS` y résout réellement.

use runique::utils::constante::ADMIN_MESSAGE_KEYS;
use serde_json::Value;

const LANGS: &[(&str, &str)] = &[
    ("fr", include_str!("../../runique_i18n/src/locales/fr/admin.json")),
    ("en", include_str!("../../runique_i18n/src/locales/en/admin.json")),
    ("it", include_str!("../../runique_i18n/src/locales/it/admin.json")),
    ("es", include_str!("../../runique_i18n/src/locales/es/admin.json")),
    ("de", include_str!("../../runique_i18n/src/locales/de/admin.json")),
    ("pt", include_str!("../../runique_i18n/src/locales/pt/admin.json")),
    ("ja", include_str!("../../runique_i18n/src/locales/ja/admin.json")),
    ("zh", include_str!("../../runique_i18n/src/locales/zh/admin.json")),
    ("ru", include_str!("../../runique_i18n/src/locales/ru/admin.json")),
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
                // Le fichier du thème `admin` : remis sous sa clé, pour résoudre
                // les clés telles que le code les écrit (`admin.…`).
                serde_json::json!({
                    "admin": serde_json::from_str::<Value>(raw)
                        .unwrap_or_else(|e| panic!("{code}/admin.json invalide : {e}")),
                }),
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
