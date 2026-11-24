//! The language of one request: what a browser's `Accept-Language` header
//! picks among the languages loaded, and the language held for the time of
//! one task, over the application's own.
use runique_i18n::{Lang, current_lang, keys, load_project, set_lang, t, with_lang};
use serial_test::serial;
use std::{fs, future::Future, path::Path};

const REQUIRED: &str = keys::forms::error::REQUIRED;

fn asked(header: &str) -> Option<Lang> {
    Lang::from_accept_language(header)
}

fn run<F: Future>(work: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(work)
}

// The first language of the header that is loaded, the others skipped.
#[test]
#[serial]
fn the_header_picks_the_preferred_language_among_the_loaded_ones() {
    assert_eq!(asked("fr"), Some(Lang::Fr));
    assert_eq!(asked("sv, fr;q=0.8, en;q=0.5"), Some(Lang::Fr));
    assert_eq!(asked("sv, da;q=0.9, de;q=0.1"), Some(Lang::De));
}

// Nothing loaded is asked for: the caller keeps its own language.
#[test]
#[serial]
fn a_header_naming_no_loaded_language_picks_none() {
    for header in ["sv, da", "", "*", ",", ";", "sv;q=0.5"] {
        assert_eq!(asked(header), None, "{header:?}");
    }
}

// The weight decides, not the place; at the same weight, the place does.
#[test]
#[serial]
fn the_weight_comes_before_the_order_of_the_header() {
    assert_eq!(asked("en;q=0.5, fr;q=0.9"), Some(Lang::Fr));
    assert_eq!(asked("fr;q=0.5, en;q=0.9"), Some(Lang::En));
    assert_eq!(asked("de, fr"), Some(Lang::De));
    assert_eq!(asked("fr, de"), Some(Lang::Fr));
    assert_eq!(asked("de;q=0.7, fr;q=0.7"), Some(Lang::De));
    assert_eq!(asked("fr;q=0.999, de"), Some(Lang::De), "no weight is 1");
    assert_eq!(asked("fr;q=0.999, de;q=1.000"), Some(Lang::De));
    assert_eq!(asked("fr;q=0.002, de;q=0.001"), Some(Lang::Fr));
}

// `q=0` is a refusal: the language is not picked, even alone.
#[test]
#[serial]
fn a_language_weighted_zero_is_refused() {
    assert_eq!(asked("fr;q=0"), None);
    assert_eq!(asked("fr;q=0.000"), None);
    assert_eq!(asked("fr;q=0, de;q=0.001"), Some(Lang::De));
    assert_eq!(asked("fr;q=0.001"), Some(Lang::Fr), "the smallest weight");
}

// A weight that isn't one drops its entry: nothing is guessed from it.
#[test]
#[serial]
fn an_entry_with_a_wrong_weight_is_dropped() {
    for wrong in [
        "q=abc", "q=1.5", "q=1.001", "q=2", "q=-1", "q=0.1234", "q=", "q=.5", "q=NaN", "q=+1",
        "q=0,5", "level=1", "",
    ] {
        let header = format!("fr;{wrong}, de;q=0.1");
        assert_eq!(asked(&header), Some(Lang::De), "{header}");
        assert_eq!(asked(&format!("fr;{wrong}")), None, "fr;{wrong}");
    }
    for right in [
        "q=1", "q=1.", "q=1.0", "q=1.000", "q=0.5", " q=0.5 ", "q= 0.5",
    ] {
        let header = format!("fr;{right}, de;q=0.1");
        assert_eq!(asked(&header), Some(Lang::Fr), "{header}");
    }
}

// Written as browsers and systems write it: case, underscore, spaces.
#[test]
#[serial]
fn the_header_is_read_whatever_its_case_and_spacing() {
    assert_eq!(asked("FR-fr"), Some(Lang::Fr));
    assert_eq!(asked("fr_CA"), Some(Lang::Fr));
    assert_eq!(asked("  de  ;q=0.5 ,  fr  "), Some(Lang::Fr));
}

// A regional variant nobody declares counts as its language, here only:
// `Lang::from` keeps answering English for a name it doesn't know.
#[test]
#[serial]
fn an_undeclared_regional_variant_counts_as_its_language() {
    assert_eq!(asked("fr-LU"), Some(Lang::Fr));
    assert_eq!(asked("de-LI, fr;q=0.9"), Some(Lang::De));
    assert_eq!(asked("sv-SE"), None);
    assert_eq!(Lang::from("fr-LU"), Lang::En);
}

// Only the first sixteen entries are read: the seventeenth picks nothing.
#[test]
#[serial]
fn only_the_first_entries_of_the_header_are_read() {
    let unknown = |count: usize| vec!["sv"; count].join(", ");
    assert_eq!(asked(&format!("{}, fr", unknown(15))), Some(Lang::Fr));
    assert_eq!(asked(&format!("{}, fr", unknown(16))), None);
}

// A language of the project is picked like one of Runique's, by its code,
// its aliases and its undeclared variants.
#[test]
#[serial]
fn the_header_picks_a_language_of_the_project() {
    let root = std::env::temp_dir().join(format!("rq_request_lang_{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let dir = root.join("locales");
    fs::create_dir_all(dir.join("nl")).unwrap();
    fs::write(
        dir.join("nl/lang.json"),
        r#"{ "name": "Nederlands", "aliases": ["nl-be"], "fallback": "fr" }"#,
    )
    .unwrap();
    fs::write(
        dir.join("nl/forms.json"),
        r#"{ "error": { "required": "Verplicht" } }"#,
    )
    .unwrap();
    load_project(&dir).unwrap();

    let dutch = Lang::from("nl");
    let picked = [
        asked("nl"),
        asked("nl-BE, fr;q=0.8"),
        asked("nl-SR"),
        asked("sv, nl;q=0.2"),
    ];
    let message = run(with_lang(dutch, async { t(REQUIRED).into_owned() }));

    let _ = fs::remove_dir_all(&root);
    load_project(Path::new("/nonexistent/runique/locales")).unwrap();

    assert_ne!(dutch, Lang::En);
    assert_eq!(picked, [Some(dutch); 4]);
    assert_eq!(message, "Verplicht");
    assert_eq!(asked("nl"), None, "the folder is gone, the language too");
}

// Inside, the language given; outside, before and after, the application's.
#[test]
#[serial]
fn a_task_answers_in_its_own_language_and_the_application_keeps_its_own() {
    set_lang(Lang::De);
    let german = t(REQUIRED).into_owned();

    let (inside, message) = run(with_lang(Lang::Fr, async {
        (current_lang(), t(REQUIRED).into_owned())
    }));
    let after = current_lang();
    set_lang(Lang::En);

    assert_eq!(inside, Lang::Fr);
    assert_eq!(message, Lang::Fr.get(REQUIRED));
    assert_ne!(message, german);
    assert_eq!(after, Lang::De, "the application's language is untouched");
}

// The language follows its task across a pause, and two tasks running in
// turn on the same thread each keep theirs.
#[test]
#[serial]
fn two_tasks_in_turn_each_keep_their_language() {
    set_lang(Lang::En);
    let in_lang = |lang: Lang| {
        with_lang(lang, async {
            let before = current_lang();
            tokio::task::yield_now().await;
            (before, current_lang())
        })
    };
    let (french, german, outside) = run(async {
        let french = tokio::spawn(in_lang(Lang::Fr));
        let german = tokio::spawn(in_lang(Lang::De));
        tokio::task::yield_now().await;
        (french.await.unwrap(), german.await.unwrap(), current_lang())
    });

    assert_eq!(french, (Lang::Fr, Lang::Fr));
    assert_eq!(german, (Lang::De, Lang::De));
    assert_eq!(outside, Lang::En);
}

// The inner language wins, and the outer one is back after it.
#[test]
#[serial]
fn a_language_inside_another_gives_it_back() {
    set_lang(Lang::En);
    let seen = run(with_lang(Lang::Fr, async {
        let inner = with_lang(Lang::De, async { current_lang() }).await;
        (inner, current_lang())
    }));
    assert_eq!(seen, (Lang::De, Lang::Fr));
}

// A task spawned from the request is another task: the application's language.
#[test]
#[serial]
fn a_spawned_task_is_back_to_the_applications_language() {
    set_lang(Lang::En);
    let (spawned, inside) = run(with_lang(Lang::Fr, async {
        let spawned = tokio::spawn(async { current_lang() }).await.unwrap();
        (spawned, current_lang())
    }));
    assert_eq!(spawned, Lang::En);
    assert_eq!(inside, Lang::Fr);
}
