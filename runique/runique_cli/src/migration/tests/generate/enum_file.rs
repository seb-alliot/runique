use crate::db::DatabaseEngine;
use crate::migration::tests::*;

const PG: DatabaseEngine = DatabaseEngine::PostgreSQL;
/// The engines where an enum column is text.
const TEXT: [DatabaseEngine; 2] = [DatabaseEngine::MySQL, DatabaseEngine::SQLite];

const RUNTIME: &str = "runique_cli::migration::runtime::";

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn places(of: &[(&str, &str)]) -> Vec<EnumPlace> {
    of.iter()
        .map(|(table, column)| EnumPlace {
            table: table.to_string(),
            column: column.to_string(),
        })
        .collect()
}

fn altered() -> EnumChanges {
    EnumChanges {
        name: "genre".into(),
        places: places(&[("books", "genre"), ("books_moods", "value")]),
        ..EnumChanges::default()
    }
}

/// The two bodies of a generated file, `up` then `down`.
fn bodies(file: &str) -> (String, String) {
    let (_, rest) = file.split_once("async fn up(").expect("an up");
    let (up, down) = rest.split_once("async fn down(").expect("a down");
    (up.to_string(), down.to_string())
}

/// The calls to the runtime a body makes, in the order it makes them.
fn calls(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(RUNTIME))
        .map(str::to_string)
        .collect()
}

// Whatever it holds, the file is Rust a migration crate compiles.
fn assert_rust(file: &str) {
    syn::parse_file(file).unwrap_or_else(|error| panic!("{error}\n{file}"));
}

// ── PostgreSQL: the enum is a type of its own ───────────────────────────────

fn created() -> EnumChanges {
    EnumChanges {
        name: "genre".into(),
        change: EnumChange::Created,
        values: strings(&["Roman", "Policier"]),
        places: places(&[("books", "genre")]),
        ..EnumChanges::default()
    }
}

#[test]
fn postgres_a_created_enum_is_created_with_its_values_and_dropped_by_down() {
    let file = generate_enum_file(&created(), &PG);
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [r#"create_enum_type(manager, "genre", &["Roman", "Policier"]).await?;"#]
    );
    assert_eq!(
        calls(&down),
        [r#"drop_enum_type(manager, "genre").await?;"#]
    );
    assert_rust(&file);
}

// One call for the enum, with every place: never one per table.
#[test]
fn a_renamed_value_is_one_call_with_every_place() {
    let file = generate_enum_file(
        &EnumChanges {
            renamed_values: vec![("Roman".into(), "Novel".into())],
            ..altered()
        },
        &PG,
    );
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [
            r#"rename_enum_value(manager, "genre", "Roman", "Novel", &[("books", "genre"), ("books_moods", "value")]).await?;"#
        ]
    );
    // `down`: the same call, its two values the other way round.
    assert_eq!(
        calls(&down),
        [
            r#"rename_enum_value(manager, "genre", "Novel", "Roman", &[("books", "genre"), ("books_moods", "value")]).await?;"#
        ]
    );
    assert_rust(&file);
}

// Several renames: `down` takes them back last first.
#[test]
fn down_takes_the_renames_back_in_reverse() {
    let file = generate_enum_file(
        &EnumChanges {
            renamed_values: vec![("a".into(), "x".into()), ("c".into(), "z".into())],
            places: places(&[("t", "c")]),
            ..altered()
        },
        &PG,
    );
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [
            r#"rename_enum_value(manager, "genre", "a", "x", &[("t", "c")]).await?;"#,
            r#"rename_enum_value(manager, "genre", "c", "z", &[("t", "c")]).await?;"#,
        ]
    );
    assert_eq!(
        calls(&down),
        [
            r#"rename_enum_value(manager, "genre", "z", "c", &[("t", "c")]).await?;"#,
            r#"rename_enum_value(manager, "genre", "x", "a", &[("t", "c")]).await?;"#,
        ]
    );
}

// Added in `up`; `down` has nothing to run and doesn't name its manager.
#[test]
fn postgres_an_added_value_is_added_and_stays_on_the_way_down() {
    let file = generate_enum_file(
        &EnumChanges {
            added_values: strings(&["Essai", "Conte"]),
            ..altered()
        },
        &PG,
    );
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [
            r#"add_enum_value(manager, "genre", "Essai").await?;"#,
            r#"add_enum_value(manager, "genre", "Conte").await?;"#,
        ]
    );
    assert!(calls(&down).is_empty(), "{down}");
    assert!(up.starts_with("&self, manager: &SchemaManager"), "{up}");
    assert!(
        down.starts_with("&self, _manager: &SchemaManager"),
        "{down}"
    );
    assert_rust(&file);
}

// Not migrated: `up` says so and runs nothing. `down` gives the type the
// value again, which is nothing when it was never taken out.
#[test]
fn postgres_a_value_taken_out_is_a_warning_and_down_gives_it_back() {
    let file = generate_enum_file(
        &EnumChanges {
            dropped_values: strings(&["Essai"]),
            ..altered()
        },
        &PG,
    );
    let (up, down) = bodies(&file);

    assert!(calls(&up).is_empty(), "{up}");
    assert!(
        up.contains(r#"// WARNING: value "Essai" removed from "genre""#),
        "{up}"
    );
    assert!(up.starts_with("&self, _manager: &SchemaManager"), "{up}");
    assert_eq!(
        calls(&down),
        [r#"add_enum_value(manager, "genre", "Essai").await?;"#]
    );
    assert_rust(&file);
}

// Renames first: a value takes its new name before the list changes around
// it. Then what is added, then what the file only warns about. `down` runs
// that backwards.
#[test]
fn postgres_renames_come_before_additions_and_warnings_last() {
    let file = generate_enum_file(
        &EnumChanges {
            // Given in another order than the one they run in.
            dropped_values: strings(&["Essai"]),
            added_values: strings(&["Drama"]),
            renamed_values: vec![("Roman".into(), "Novel".into())],
            places: places(&[("books", "genre")]),
            ..altered()
        },
        &PG,
    );
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [
            r#"rename_enum_value(manager, "genre", "Roman", "Novel", &[("books", "genre")]).await?;"#,
            r#"add_enum_value(manager, "genre", "Drama").await?;"#,
        ]
    );
    assert!(
        up.find("add_enum_value").unwrap() < up.find("// WARNING").unwrap(),
        "{up}"
    );
    assert_eq!(
        calls(&down),
        [
            r#"add_enum_value(manager, "genre", "Essai").await?;"#,
            r#"rename_enum_value(manager, "genre", "Novel", "Roman", &[("books", "genre")]).await?;"#,
        ]
    );
    assert_rust(&file);
}

// The enum is named as the database names it, whatever the case it is given
// in: the type and every call agree.
#[test]
fn the_enum_is_named_in_lowercase_in_every_call() {
    let renamed = EnumChanges {
        name: "OrderStatus".into(),
        renamed_values: vec![("a".into(), "b".into())],
        added_values: strings(&["c"]),
        ..altered()
    };
    let file = generate_enum_file(&renamed, &PG);
    assert!(!file.contains("OrderStatus"), "{file}");
    assert_eq!(
        file.matches(r#"(manager, "orderstatus","#).count(),
        3,
        "{file}"
    );
    let file = generate_enum_file(
        &EnumChanges {
            name: "OrderStatus".into(),
            ..created()
        },
        &PG,
    );
    assert_eq!(
        file.matches(r#"(manager, "orderstatus""#).count(),
        2,
        "{file}"
    );
}

// ── An enum renamed ──────────────────────────────────────────────────────────

fn renamed() -> EnumChanges {
    EnumChanges {
        name: "Category".into(),
        change: EnumChange::Renamed,
        renamed_from: Some("Genre".into()),
        ..created()
    }
}

// Renamed in place, by the name the database has each under; `down` gives
// the former name back.
#[test]
fn postgres_a_renamed_enum_takes_its_new_name_and_down_gives_the_former_back() {
    let file = generate_enum_file(&renamed(), &PG);
    let (up, down) = bodies(&file);

    assert_eq!(
        calls(&up),
        [r#"rename_enum_type(manager, "genre", "category").await?;"#]
    );
    assert_eq!(
        calls(&down),
        [r#"rename_enum_type(manager, "category", "genre").await?;"#]
    );
    // Its values and its places are not this file's business.
    assert!(!file.contains("Roman") && !file.contains("books"), "{file}");
    assert_rust(&file);
}

// No object carries the name of an enum there: the file runs nothing, and is
// what records the enum under its new name.
#[test]
fn where_the_column_is_text_a_renamed_enum_runs_nothing() {
    for engine in &TEXT {
        let file = generate_enum_file(&renamed(), engine);
        let (up, down) = bodies(&file);

        assert!(calls(&up).is_empty() && calls(&down).is_empty(), "{file}");
        assert!(!file.contains("rename_enum_type"), "{file}");
        assert!(up.contains("// Nothing to run on this engine"), "{up}");
        assert_rust(&file);
    }
}

// A rename that doesn't say from what has nothing to rename.
#[test]
fn a_rename_without_a_former_name_runs_nothing() {
    let file = generate_enum_file(
        &EnumChanges {
            renamed_from: None,
            ..renamed()
        },
        &PG,
    );
    let (up, down) = bodies(&file);
    assert!(calls(&up).is_empty() && calls(&down).is_empty(), "{file}");
    assert_rust(&file);
}

// ── MySQL and SQLite: the column is text ─────────────────────────────────────

const NOTHING: &str = r#"// Nothing to run on this engine for the enum "genre""#;

// No type to create there: the file runs nothing, says why, and still
// exists, since applying it is what records the enum.
#[test]
fn where_the_column_is_text_a_created_enum_runs_nothing() {
    for engine in &TEXT {
        let file = generate_enum_file(&created(), engine);
        let (up, down) = bodies(&file);

        assert!(calls(&up).is_empty() && calls(&down).is_empty(), "{file}");
        assert!(!file.contains("create_enum_type") && !file.contains("drop_enum_type"));
        assert!(up.contains(NOTHING), "{up}");
        assert!(up.starts_with("&self, _manager: &SchemaManager"), "{up}");
        assert!(
            down.starts_with("&self, _manager: &SchemaManager"),
            "{down}"
        );
        assert_rust(&file);
    }
}

// A text column takes the new value as it is.
#[test]
fn where_the_column_is_text_an_added_value_runs_nothing() {
    for engine in &TEXT {
        let file = generate_enum_file(
            &EnumChanges {
                added_values: strings(&["Essai"]),
                ..altered()
            },
            engine,
        );
        let (up, down) = bodies(&file);

        assert!(calls(&up).is_empty() && calls(&down).is_empty(), "{file}");
        assert!(!file.contains("add_enum_value"), "{file}");
        assert!(up.contains(NOTHING), "{up}");
        assert!(up.starts_with("&self, _manager: &SchemaManager"), "{up}");
        assert_rust(&file);
    }
}

// The same enum under PostgreSQL has something to run: the engine decides
// what the file holds, and MySQL and SQLite hold the same.
#[test]
fn the_engine_decides_what_the_file_holds() {
    let added = EnumChanges {
        added_values: strings(&["Essai"]),
        ..altered()
    };
    for changes in [created(), added] {
        let on_postgres = generate_enum_file(&changes, &PG);
        assert!(!on_postgres.contains("Nothing to run"), "{on_postgres}");
        for engine in &TEXT {
            assert_ne!(generate_enum_file(&changes, engine), on_postgres);
        }
        assert_eq!(
            generate_enum_file(&changes, &TEXT[0]),
            generate_enum_file(&changes, &TEXT[1])
        );
    }
}

// A renamed value has rows to write there: the call is the one PostgreSQL
// gets, with every place, and so is its way back.
#[test]
fn a_renamed_value_is_the_same_file_on_every_engine() {
    let renamed = EnumChanges {
        renamed_values: vec![("Roman".into(), "Novel".into())],
        ..altered()
    };
    let on_postgres = generate_enum_file(&renamed, &PG);
    for engine in &TEXT {
        let file = generate_enum_file(&renamed, engine);
        assert_eq!(file, on_postgres);
        assert!(!file.contains("Nothing to run"), "{file}");
    }
}

// A value taken out: the same warning as under PostgreSQL, and no way back
// to run, a text column having no list to give the value to.
#[test]
fn where_the_column_is_text_a_value_taken_out_is_a_warning_only() {
    for engine in &TEXT {
        let file = generate_enum_file(
            &EnumChanges {
                dropped_values: strings(&["Essai"]),
                ..altered()
            },
            engine,
        );
        let (up, down) = bodies(&file);

        assert!(calls(&up).is_empty() && calls(&down).is_empty(), "{file}");
        assert!(
            up.contains(r#"// WARNING: value "Essai" removed from "genre""#),
            "{up}"
        );
        assert!(!up.contains("Nothing to run"), "{up}");
        assert!(
            down.starts_with("&self, _manager: &SchemaManager"),
            "{down}"
        );
        assert_rust(&file);
    }
}

// Everything at once where the column is text: the rename runs, and it is
// all that runs, on the way up and on the way down.
#[test]
fn where_the_column_is_text_only_the_rename_runs() {
    for engine in &TEXT {
        let file = generate_enum_file(
            &EnumChanges {
                renamed_values: vec![("Roman".into(), "Novel".into())],
                added_values: strings(&["Drama"]),
                dropped_values: strings(&["Essai"]),
                places: places(&[("books", "genre")]),
                ..altered()
            },
            engine,
        );
        let (up, down) = bodies(&file);

        assert_eq!(
            calls(&up),
            [
                r#"rename_enum_value(manager, "genre", "Roman", "Novel", &[("books", "genre")]).await?;"#
            ]
        );
        assert_eq!(
            calls(&down),
            [
                r#"rename_enum_value(manager, "genre", "Novel", "Roman", &[("books", "genre")]).await?;"#
            ]
        );
        assert!(!file.contains("add_enum_value"), "{file}");
        assert_rust(&file);
    }
}

// ── What a value may hold ────────────────────────────────────────────────────

// A value is written as a Rust string, whatever it holds: a quote or a
// backslash in it can't end the string and run as code.
#[test]
fn a_value_is_written_as_a_string_whatever_it_holds() {
    let hostile = r#"a"); panic!("x"#;
    for changes in [
        EnumChanges {
            values: strings(&[hostile, "back\\slash", "l'été"]),
            ..created()
        },
        EnumChanges {
            renamed_values: vec![(hostile.into(), "back\\slash".into())],
            added_values: strings(&[hostile]),
            dropped_values: strings(&[hostile]),
            ..altered()
        },
    ] {
        for engine in [&PG, &TEXT[0], &TEXT[1]] {
            let file = generate_enum_file(&changes, engine);
            assert_rust(&file);
            assert!(!file.contains("panic!(\"x\""), "{file}");
        }
        let on_postgres = generate_enum_file(&changes, &PG);
        assert!(on_postgres.contains(r#"a\"); panic!(\"x"#), "{on_postgres}");
    }
}

// The same goes for the names of the places.
#[test]
fn a_place_is_written_as_strings_whatever_its_names_hold() {
    let file = generate_enum_file(
        &EnumChanges {
            renamed_values: vec![("a".into(), "b".into())],
            places: places(&[("t\"; drop", "c\\")]),
            ..altered()
        },
        &PG,
    );
    assert_rust(&file);
    assert!(file.contains(r#"("t\"; drop", "c\\")"#), "{file}");
}

// A value or a name in a comment stays on its line: a line break in it
// can't end the comment and run as code.
#[test]
fn what_is_written_in_a_comment_stays_on_its_line() {
    let hostile = "x\npanic!();";
    for engine in &TEXT {
        let dropped = generate_enum_file(
            &EnumChanges {
                dropped_values: strings(&[hostile]),
                ..altered()
            },
            engine,
        );
        let named = generate_enum_file(
            &EnumChanges {
                name: hostile.into(),
                ..created()
            },
            engine,
        );
        for file in [dropped, named] {
            let (up, _) = bodies(&file);
            assert!(body_runs_nothing(&up), "{up}");
            assert_rust(&file);
        }
    }
}

/// True when a body holds comments and its closing `Ok(())` only.
fn body_runs_nothing(body: &str) -> bool {
    body.lines()
        .skip(1)
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .all(|line| line == "Ok(())" || line == "}")
}
