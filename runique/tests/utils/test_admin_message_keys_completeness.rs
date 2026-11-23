//! Test — complétude i18n de `ADMIN_MESSAGE_KEYS` sur les 9 langues.
//!
//! Deuxième test du chantier "détection des bugs muets" (cf. mémoire projet
//! `project_tests_detection_bugs_muets`). Motivation : `t()`/`Lang::get()`
//! retombe silencieusement sur l'anglais puis sur la clé brute si absente —
//! aucun test de rendu (route crawl inclus) ne peut voir une clé manquante
//! dans une langue tant qu'elle existe en anglais. Ce test lit les fichiers `admin/*.json`
//! de chacune des 9 langues directement (sur le disque, contournant le fallback de `Lang::get`)
//! et vérifie que chaque clé de `ADMIN_MESSAGE_KEYS` y résout réellement.

use runique::utils::constante::ADMIN_MESSAGE_KEYS;
use serde_json::Value;

const LANGS: &[&str] = &["fr", "en", "it", "es", "de", "pt", "ja", "zh", "ru"];

/// Les fichiers `admin/<écran>.json` d'une langue, remis sous leur clé
/// (`admin.<écran>`) pour résoudre les clés telles que le code les écrit.
fn admin_messages(code: &str) -> Value {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("runique_i18n/src/locales")
        .join(code)
        .join("admin");
    let mut screens = serde_json::Map::new();
    for entry in std::fs::read_dir(&folder).unwrap_or_else(|e| panic!("{}: {e}", folder.display()))
    {
        let path = entry.unwrap().path();
        let screen = path.file_stem().unwrap().to_string_lossy().into_owned();
        let raw = std::fs::read_to_string(&path).unwrap();
        let json: Value = serde_json::from_str(&raw)
            .unwrap_or_else(|e| panic!("{} invalide : {e}", path.display()));
        screens.insert(screen, json);
    }
    serde_json::json!({ "admin": screens })
}

/// Résout une clé pointée (runique_i18n::keys::admin::login::label::TITLE) dans un JSON imbriqué, sans
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
        .map(|code| (*code, admin_messages(code)))
        .collect();

    let mut missing: Vec<String> = Vec::new();
    for (_variable, key) in ADMIN_MESSAGE_KEYS {
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

// Deux messages sous la même variable de template : le second écraserait le
// premier dans le contexte, sans erreur.
#[test]
fn every_admin_template_variable_is_unique() {
    let mut seen = std::collections::HashSet::new();
    for (variable, key) in ADMIN_MESSAGE_KEYS {
        assert!(
            seen.insert(*variable),
            "{variable} est donnée deux fois ({key})"
        );
        assert!(key.starts_with("admin."), "{key}");
        assert!(variable.starts_with("admin_"), "{variable}");
    }
}
