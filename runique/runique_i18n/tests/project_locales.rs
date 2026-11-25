//! A project's own `locales/` folder: a language Runique doesn't ship, a
//! message of Runique replaced, subjects of the project's own, and every way
//! the folder can be wrong.
//!
//! The languages are the process's: each test loads its own folder, one at a
//! time, and puts Runique's languages back.
use runique_i18n::{Lang, LocaleError, keys, load_project, set_lang, t};
use serial_test::serial;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

/// A project folder, removed with the test, Runique's languages put back.
struct Project(PathBuf);

impl Project {
    /// A `locales` folder holding `files`, by their path in it.
    fn with(files: &[(&str, &str)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "rq_locales_{}_{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = fs::remove_dir_all(&root);
        let project = Self(root);
        fs::create_dir_all(project.dir()).unwrap();
        for (path, content) in files {
            let file = project.dir().join(path);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, content).unwrap();
        }
        project
    }

    fn dir(&self) -> PathBuf {
        self.0.join("locales")
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
        let _ = load_project(Path::new("/nonexistent/runique/locales"));
        set_lang(Lang::En);
    }
}

const REQUIRED: &str = keys::forms::error::REQUIRED;
const TOO_SHORT: &str = keys::forms::error::TOO_SHORT;

// No folder: nothing to read, Runique's nine languages and nothing else.
#[test]
#[serial]
fn without_a_folder_runique_keeps_its_own_languages() {
    let project = Project::with(&[]);
    fs::remove_dir_all(project.dir()).unwrap();

    let report = load_project(&project.dir()).unwrap();

    assert!(report.languages.is_empty());
    assert_eq!(Lang::available().len(), 9);
    assert_eq!(Lang::from("nl"), Lang::En, "an unknown name is English");
    assert_ne!(Lang::Fr.get(REQUIRED), REQUIRED);
}

// A folder of another name is a language: found by its code and its aliases,
// with its own messages, and the rest in the language it falls back on.
#[test]
#[serial]
fn a_folder_of_another_name_is_a_new_language() {
    let project = Project::with(&[
        (
            "nl/lang.json",
            r#"{ "name": "Nederlands", "aliases": ["nl-NL", "nl_BE"], "fallback": "fr" }"#,
        ),
        (
            "nl/forms.json",
            r#"{ "error": { "required": "Dit veld is verplicht" } }"#,
        ),
    ]);

    let report = load_project(&project.dir()).unwrap();

    let dutch = Lang::from("nl");
    assert!(matches!(dutch, Lang::Other(_)), "{dutch:?}");
    assert_eq!(dutch.code(), "nl");
    assert_eq!(dutch.name(), "Nederlands");
    for written in ["NL", "nl-be", "nl_NL.UTF-8", " nl "] {
        assert_eq!(Lang::from(written), dutch, "{written:?}");
    }
    assert_eq!(Lang::available().len(), 10);
    assert_eq!(Lang::available().last(), Some(&dutch));

    // Its own message, then French (its fallback), never English first.
    assert_eq!(dutch.get(REQUIRED), "Dit veld is verplicht");
    assert_eq!(dutch.get(TOO_SHORT), Lang::Fr.get(TOO_SHORT));
    assert_ne!(dutch.get(TOO_SHORT), Lang::En.get(TOO_SHORT));
    // The others are untouched.
    assert_ne!(Lang::Fr.get(REQUIRED), "Dit veld is verplicht");

    // As the language in use.
    set_lang(dutch);
    assert_eq!(t(REQUIRED), "Dit veld is verplicht");

    let nl = &report.languages[0];
    assert_eq!((nl.code.as_str(), nl.new, nl.messages), ("nl", true, 1));
    assert!(nl.unknown.is_empty() && nl.mismatched.is_empty());
    let forms = nl
        .missing
        .iter()
        .find(|(subject, _)| subject == "forms")
        .unwrap();
    assert!(
        forms.1 > 10,
        "every other message of `forms` is missing: {forms:?}"
    );
    assert!(nl.missing.iter().any(|(subject, _)| subject == "admin"));
}

// Without a `lang.json` the language is called by its code and falls back on
// English.
#[test]
#[serial]
fn a_language_without_its_description_falls_back_on_english() {
    let project = Project::with(&[(
        "nl/forms.json",
        r#"{ "error": { "required": "Verplicht" } }"#,
    )]);

    load_project(&project.dir()).unwrap();

    let dutch = Lang::from("nl");
    assert_eq!(dutch.name(), "nl");
    assert_eq!(dutch.get(REQUIRED), "Verplicht");
    assert_eq!(dutch.get(TOO_SHORT), Lang::En.get(TOO_SHORT));
    assert_eq!(dutch.get("nothing.has.this"), "nothing.has.this");
}

// A folder named after one of Runique's languages adds to it: a message of
// the same key comes first, the others stay Runique's, in that language only.
#[test]
#[serial]
fn a_project_replaces_a_message_of_runique_in_one_language() {
    let before = Lang::Fr.get(TOO_SHORT);
    let project = Project::with(&[
        (
            "fr/forms.json",
            r#"{ "error": { "required": "À remplir" } }"#,
        ),
        (
            "fr/admin/login.json",
            r#"{ "label": { "title": "Entrée" } }"#,
        ),
        // The description of a language of Runique's is Runique's.
        ("fr/lang.json", r#"{ "name": "Autre", "fallback": "it" }"#),
    ]);

    let report = load_project(&project.dir()).unwrap();

    assert_eq!(Lang::Fr.get(REQUIRED), "À remplir");
    assert_eq!(Lang::Fr.get(keys::admin::login::label::TITLE), "Entrée");
    assert_eq!(Lang::Fr.get(TOO_SHORT), before);
    assert_ne!(Lang::En.get(REQUIRED), "À remplir");
    assert_eq!(Lang::Fr.name(), "Français");
    assert_eq!(Lang::available().len(), 9);

    let fr = &report.languages[0];
    assert_eq!((fr.code.as_str(), fr.new, fr.messages), ("fr", false, 2));
    assert!(
        fr.missing.is_empty(),
        "a language of Runique misses nothing"
    );
}

// A subject Runique doesn't have is the project's: its keys are asked for
// like any other, and aren't reported as mistakes.
#[test]
#[serial]
fn a_project_has_subjects_of_its_own() {
    let project = Project::with(&[
        (
            "fr/shop.json",
            r#"{ "title": "Boutique", "cart": { "empty": "Panier vide" } }"#,
        ),
        ("en/shop.json", r#"{ "title": "Shop" }"#),
    ]);

    let report = load_project(&project.dir()).unwrap();

    assert_eq!(Lang::Fr.get("shop.title"), "Boutique");
    assert_eq!(Lang::Fr.get("shop.cart.empty"), "Panier vide");
    assert_eq!(Lang::En.get("shop.title"), "Shop");
    // Not in English, the last fallback: the key itself.
    assert_eq!(Lang::En.get("shop.cart.empty"), "shop.cart.empty");
    // Italian has none of its own: English's.
    assert_eq!(Lang::It.get("shop.title"), "Shop");
    assert!(report.languages.iter().all(|l| l.unknown.is_empty()));
}

// What nothing will ever ask for, and what would drop a parameter, is said.
#[test]
#[serial]
fn a_misspelt_key_and_a_wrong_number_of_parameters_are_reported() {
    let project = Project::with(&[(
        "fr/forms.json",
        r#"{ "error": { "requird": "Oups", "too_short": "Trop court", "required": "Requis" } }"#,
    )]);

    let report = load_project(&project.dir()).unwrap();

    let fr = &report.languages[0];
    assert_eq!(fr.unknown, ["forms.error.requird"]);
    assert_eq!(fr.mismatched, ["forms.error.too_short"]);
    // Reported, and still loaded: it is the project's choice.
    assert_eq!(Lang::Fr.get(TOO_SHORT), "Trop court");
}

/// The error of loading a folder holding `files`.
fn refused(files: &[(&str, &str)]) -> LocaleError {
    let project = Project::with(files);
    let error = load_project(&project.dir()).unwrap_err();
    // Nothing of a refused folder is kept.
    assert_eq!(Lang::available().len(), 9, "{error}");
    error
}

#[test]
#[serial]
fn a_file_that_is_not_json_is_refused_by_its_name() {
    let error = refused(&[("fr/forms.json", "{ not json")]);
    assert!(
        matches!(&error, LocaleError::Invalid { path, .. } if path.ends_with("fr/forms.json")),
        "{error:?}"
    );
    assert!(error.to_string().contains("forms.json"), "{error}");

    for not_an_object in ["[]", "\"text\"", "3", "null"] {
        let error = refused(&[("fr/forms.json", not_an_object)]);
        assert!(
            matches!(error, LocaleError::Invalid { .. }),
            "{not_an_object}: {error:?}"
        );
    }
}

#[test]
#[serial]
fn a_value_that_is_not_a_text_is_refused_by_its_key() {
    for wrong in ["3", "true", "null", "[\"a\"]"] {
        let error = refused(&[(
            "fr/forms.json",
            &format!(r#"{{ "error": {{ "required": {wrong} }} }}"#),
        )]);
        assert_eq!(
            error,
            LocaleError::NotText {
                path: error_path(&error),
                key: "error.required".to_string()
            },
            "{wrong}"
        );
    }
}

fn error_path(error: &LocaleError) -> PathBuf {
    match error {
        LocaleError::NotText { path, .. } => path.clone(),
        other => panic!("{other:?}"),
    }
}

// A folder is a language only under a code: anything else is refused rather
// than taken for one.
#[test]
#[serial]
fn a_folder_that_is_not_named_by_a_code_is_refused() {
    for name in [
        "FR",
        "Nl",
        "f",
        "français",
        "fr_FR",
        "fr-",
        "-fr",
        "fr--be",
        "1fr",
        "a-very-long-language-name",
    ] {
        let error = refused(&[(&format!("{name}/forms.json"), "{}")]);
        assert!(
            matches!(error, LocaleError::NotALanguage { .. }),
            "{name}: {error:?}"
        );
    }
    // A file beside the language folders is not a language.
    let project = Project::with(&[("README.md", "notes"), ("fr/notes.txt", "not a message")]);
    let report = load_project(&project.dir()).unwrap();
    assert_eq!(report.languages.len(), 1);
    assert_eq!(report.languages[0].messages, 0);
}

#[test]
#[serial]
fn a_fallback_that_names_no_language_is_refused() {
    let error = refused(&[("nl/lang.json", r#"{ "fallback": "xx" }"#)]);
    assert_eq!(
        error,
        LocaleError::UnknownFallback {
            language: "nl".to_string(),
            fallback: "xx".to_string()
        }
    );
}

// On itself, or round in a circle: a missing message would be looked for
// without end.
#[test]
#[serial]
fn fallbacks_that_never_reach_english_are_refused() {
    let error = refused(&[("nl/lang.json", r#"{ "fallback": "nl" }"#)]);
    assert!(
        matches!(error, LocaleError::FallbackLoop { .. }),
        "{error:?}"
    );

    let error = refused(&[
        ("nl/lang.json", r#"{ "fallback": "af" }"#),
        ("af/lang.json", r#"{ "fallback": "nl" }"#),
    ]);
    assert!(
        matches!(error, LocaleError::FallbackLoop { .. }),
        "{error:?}"
    );

    // A chain that does reach it is fine, however long.
    let project = Project::with(&[
        ("nl/lang.json", r#"{ "fallback": "af" }"#),
        ("af/lang.json", r#"{ "fallback": "fr" }"#),
        ("af/forms.json", r#"{ "error": { "required": "Vereis" } }"#),
    ]);
    load_project(&project.dir()).unwrap();
    assert_eq!(Lang::from("nl").get(REQUIRED), "Vereis");
    assert_eq!(Lang::from("nl").get(TOO_SHORT), Lang::Fr.get(TOO_SHORT));
}

// One name, one language: an alias already taken would answer for the wrong
// one. A regional variant nobody declared is free to take.
#[test]
#[serial]
fn a_name_given_to_two_languages_is_refused() {
    for taken in ["fr", "FR", "af"] {
        let error = refused(&[
            ("nl/lang.json", &format!(r#"{{ "aliases": ["{taken}"] }}"#)),
            ("af/forms.json", "{}"),
        ]);
        assert!(
            matches!(error, LocaleError::SameName { .. }),
            "{taken}: {error:?}"
        );
    }
}

#[test]
#[serial]
fn a_description_with_a_wrong_field_is_refused() {
    for wrong in [
        r#"{ "name": 3 }"#,
        r#"{ "fallback": ["fr"] }"#,
        r#"{ "aliases": "nl-be" }"#,
        r#"{ "aliases": [3] }"#,
        "[]",
        "{ not json",
    ] {
        let error = refused(&[("nl/lang.json", wrong)]);
        assert!(
            matches!(&error, LocaleError::Invalid { path, .. } if path.ends_with("nl/lang.json")),
            "{wrong}: {error:?}"
        );
    }
}

// Loading again replaces: the languages of the former folder are gone, and
// the language in use, if it was one of them, is English again.
#[test]
#[serial]
fn loading_again_replaces_the_former_folder() {
    let first = Project::with(&[(
        "nl/forms.json",
        r#"{ "error": { "required": "Verplicht" } }"#,
    )]);
    load_project(&first.dir()).unwrap();
    let dutch = Lang::from("nl");
    set_lang(dutch);
    assert_eq!(runique_i18n::current_lang(), dutch);

    let second = Project::with(&[("fr/shop.json", r#"{ "title": "Boutique" }"#)]);
    load_project(&second.dir()).unwrap();

    assert_eq!(Lang::available().len(), 9);
    assert_eq!(Lang::from("nl"), Lang::En);
    assert_eq!(runique_i18n::current_lang(), Lang::En);
    assert_eq!(t(REQUIRED), Lang::En.get(REQUIRED));
    assert_eq!(
        dutch.get(REQUIRED),
        Lang::En.get(REQUIRED),
        "a language no longer loaded"
    );
    assert_eq!(dutch.code(), "en");
    assert_eq!(Lang::Fr.get("shop.title"), "Boutique");
}

// The default of the environment doesn't undo a language chosen in the code.
#[test]
#[serial]
fn a_default_language_gives_way_to_a_chosen_one() {
    let _project = Project::with(&[]);
    set_lang(Lang::It);
    runique_i18n::set_default_lang(Lang::Fr);
    assert_eq!(runique_i18n::current_lang(), Lang::It);
}

// The codes Runique's own files give: the same nine as before, by the same
// names, regional variants included.
#[test]
#[serial]
fn the_languages_of_runique_are_described_by_their_own_files() {
    let _project = Project::with(&[]);
    for (written, lang, code) in [
        ("fr", Lang::Fr, "fr"),
        ("fr-CA", Lang::Fr, "fr"),
        ("fr_FR.UTF-8", Lang::Fr, "fr"),
        ("pt-BR", Lang::Pt, "pt"),
        ("zh-TW", Lang::Zh, "zh"),
        ("ja-JP", Lang::Ja, "ja"),
        ("ru-UA", Lang::Ru, "ru"),
        ("de-AT", Lang::De, "de"),
        ("es-MX", Lang::Es, "es"),
        ("it-CH", Lang::It, "it"),
        ("en-US", Lang::En, "en"),
        ("fr-LU", Lang::Fr, "fr"),
        ("sv-SE", Lang::En, "en"),
    ] {
        assert_eq!(Lang::from(written), lang, "{written}");
        assert_eq!(Lang::from(written).code(), code, "{written}");
    }
    assert_eq!(Lang::Fr.name(), "Français");
    assert_eq!(Lang::Ja.name(), "日本語");
    assert_eq!(
        Lang::available(),
        [
            Lang::Fr,
            Lang::En,
            Lang::It,
            Lang::Es,
            Lang::De,
            Lang::Pt,
            Lang::Ja,
            Lang::Zh,
            Lang::Ru
        ]
    );
}

// Only the first part of a code counts: a regional variant is its language.
// `find` says when nothing answers, where `from` says English.
#[test]
#[serial]
fn a_code_is_its_language_whatever_its_region() {
    let _project = Project::with(&[]);
    for (written, lang) in [
        ("fr", Some(Lang::Fr)),
        ("fr-LU", Some(Lang::Fr)),
        ("fr_FR.UTF-8", Some(Lang::Fr)),
        ("PT-br", Some(Lang::Pt)),
        ("zh-Hant-TW", Some(Lang::Zh)),
        ("sv-SE", None),
        ("sv", None),
        ("", None),
        ("-fr", None),
    ] {
        assert_eq!(Lang::find(written), lang, "{written:?}");
        assert_eq!(Lang::from(written), lang.unwrap_or(Lang::En), "{written:?}");
    }
}

// A folder named after a variant of one of Runique's languages is that
// language: its messages come over Runique's, for every variant.
#[test]
#[serial]
fn a_variant_folder_completes_the_language_of_runique() {
    let project = Project::with(&[(
        "pt-BR/forms.json",
        r#"{ "error": { "required": "Campo obrigatório" } }"#,
    )]);
    let report = load_project(&project.dir()).unwrap();

    assert_eq!(Lang::available().len(), 9, "no language added");
    assert_eq!(report.languages.len(), 1);
    assert_eq!(report.languages[0].code, "pt");
    assert!(!report.languages[0].new);
    for written in ["pt", "pt-BR", "pt-PT", "pt_BR.UTF-8"] {
        assert_eq!(Lang::find(written), Some(Lang::Pt), "{written}");
    }
    assert_eq!(Lang::Pt.get(REQUIRED), "Campo obrigatório");
    assert_eq!(Lang::Pt.get(TOO_SHORT), Lang::from("pt").get(TOO_SHORT));
}

// A folder named `nl-NL`, `nl_NL` or `NL` is the language `nl`, found by its
// code and by every variant of it; a name that is no code is still refused.
#[test]
#[serial]
fn a_language_folder_is_named_by_its_language_whatever_its_region() {
    for folder in ["nl", "nl-NL", "nl_NL", "NL-be", "NL"] {
        let project = Project::with(&[(
            &format!("{folder}/forms.json"),
            r#"{ "error": { "required": "Verplicht" } }"#,
        )]);
        load_project(&project.dir()).unwrap();
        let dutch = Lang::find("nl").unwrap_or_else(|| panic!("{folder}"));
        assert_eq!(dutch.code(), "nl", "{folder}");
        assert_eq!(dutch.get(REQUIRED), "Verplicht", "{folder}");
        for written in ["nl-NL", "nl-BE", "nl_NL.UTF-8", "NL"] {
            assert_eq!(Lang::find(written), Some(dutch), "{folder}: {written}");
        }
    }
    for folder in ["N L", "nl.bak", "-nl", "nl-"] {
        let error = refused(&[(&format!("{folder}/forms.json"), "{}")]);
        assert!(
            matches!(error, LocaleError::NotALanguage { .. }),
            "{folder}: {error:?}"
        );
    }
}

// Two folders of one language would answer one for the other: refused by
// the language they share, for a new language as for one of Runique's.
#[test]
#[serial]
fn two_folders_of_one_language_are_refused() {
    for (first, second, shared) in [
        ("nl-NL", "nl-BE", "nl"),
        ("pt-BR", "pt-PT", "pt"),
        ("fr", "fr-CA", "fr"),
    ] {
        let error = refused(&[
            (&format!("{first}/forms.json"), "{}"),
            (&format!("{second}/forms.json"), "{}"),
        ]);
        assert_eq!(
            error,
            LocaleError::SameName {
                name: shared.to_string()
            },
            "{first} + {second}"
        );
    }
}
