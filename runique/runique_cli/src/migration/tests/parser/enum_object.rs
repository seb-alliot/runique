use crate::{atomic::tests::TempDir, migration::tests::*};
use std::fs;

fn enum_column(name: &str, of: &str, values: &[&str]) -> ParsedColumn {
    ParsedColumn {
        name: name.into(),
        enum_name: Some(of.into()),
        enum_string_values: values.iter().map(|value| value.to_string()).collect(),
        ..ParsedColumn::default()
    }
}

fn table(name: &str, columns: Vec<ParsedColumn>) -> ParsedSchema {
    ParsedSchema {
        table_name: name.into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..ParsedColumn::default()
        }),
        columns,
        foreign_keys: vec![],
        indexes: vec![],
    }
}

fn place(table: &str, column: &str) -> EnumPlace {
    EnumPlace {
        table: table.into(),
        column: column.into(),
    }
}

fn genre(values: &[&str]) -> ParsedEnum {
    ParsedEnum {
        name: "genre".into(),
        values: values.iter().map(|value| value.to_string()).collect(),
        places: Vec::new(),
    }
}

// ── The enums of the models ──────────────────────────────────────────────────

// One enum used by two tables and by two columns of one table: it is found
// once, with its three places in the order they were met.
#[test]
fn an_enum_is_found_once_with_every_column_that_uses_it() {
    let values = ["Roman", "Policier"];
    let schemas = [
        table(
            "books",
            vec![
                enum_column("genre", "Genre", &values),
                enum_column("former_genre", "Genre", &values),
            ],
        ),
        table("books_moods", vec![enum_column("value", "Genre", &values)]),
    ];

    assert_eq!(
        enums_of_schemas(&schemas),
        [ParsedEnum {
            places: vec![
                place("books", "genre"),
                place("books", "former_genre"),
                place("books_moods", "value"),
            ],
            ..genre(&values)
        }]
    );
}

// The database knows the enum by its name in lowercase; the enums come out
// sorted by that name whatever the order of the tables.
#[test]
fn the_enums_are_named_as_the_database_names_them_and_sorted() {
    let schemas = [
        table(
            "orders",
            vec![enum_column("status", "OrderStatus", &["Open"])],
        ),
        table("books", vec![enum_column("genre", "Genre", &["Roman"])]),
    ];

    let names: Vec<String> = enums_of_schemas(&schemas)
        .into_iter()
        .map(|found| found.name)
        .collect();
    assert_eq!(names, ["genre", "orderstatus"]);
}

// A column without stored values is not an enum of the database: a plain
// column, or an enum kept as an integer.
#[test]
fn a_column_without_stored_values_is_no_enum() {
    let plain = ParsedColumn {
        name: "title".into(),
        ..ParsedColumn::default()
    };
    let integer = ParsedColumn {
        name: "level".into(),
        col_type: crate::migration::ColumnType::Integer,
        enum_name: None,
        ..ParsedColumn::default()
    };
    assert!(enums_of_schemas(&[table("books", vec![plain, integer])]).is_empty());
    assert!(enums_of_schemas(&[]).is_empty());
}

// A column the database doesn't hold is not a place of the enum.
#[test]
fn an_ignored_column_is_not_a_place() {
    let ignored = ParsedColumn {
        ignored: true,
        ..enum_column("genre", "Genre", &["Roman"])
    };
    assert!(enums_of_schemas(&[table("books", vec![ignored])]).is_empty());
}

// An enum column without a name of its own is named after its column.
#[test]
fn an_enum_without_a_name_takes_its_column_s() {
    let unnamed = ParsedColumn {
        enum_name: None,
        ..enum_column("Mood", "", &["Calm"])
    };
    let found = enums_of_schemas(&[table("books", vec![unnamed])]);
    assert_eq!(found[0].name, "mood");
}

// ── The snapshot of an enum, written then read back ──────────────────────────

#[test]
fn a_snapshot_is_read_back_as_it_was_written() {
    for values in [
        vec!["Roman", "Policier"],
        vec!["Zeta", "alpha", "Mid"],
        vec!["one"],
        vec![
            "science fiction",
            "l'été",
            "say \"hi\"",
            "back\\slash",
            "日本",
        ],
    ] {
        let written = genre(&values);
        let read = parse_enum_snapshot(&generate_enum_snapshot_file(&written)).unwrap();
        assert_eq!(read, written, "{values:?}");
    }
}

// The places are not in the snapshot: they come from the models.
#[test]
fn a_snapshot_holds_no_place() {
    let written = ParsedEnum {
        places: vec![place("books", "genre")],
        ..genre(&["Roman"])
    };
    let read = parse_enum_snapshot(&generate_enum_snapshot_file(&written)).unwrap();
    assert_eq!(read, genre(&["Roman"]));
}

// The enum is the one given to `as_enum`: another name written in the file,
// before or after, is not it.
#[test]
fn the_name_is_the_one_the_type_is_created_under() {
    let source = r#"
        fn up() {
            manager.drop_type(Type::drop().name(Alias::new("before")).to_owned());
            manager.create_type(
                Type::create()
                    .as_enum(Alias::new("genre"))
                    .values([Alias::new("Roman")])
                    .to_owned(),
            );
            manager.drop_type(Type::drop().name(Alias::new("after")).to_owned());
        }
    "#;
    assert_eq!(parse_enum_snapshot(source).unwrap(), genre(&["Roman"]));
}

// A file that names things without creating an enum type has no enum.
#[test]
fn a_name_alone_does_not_make_an_enum() {
    let source = r#"
        fn up() {
            manager.drop_type(Type::drop().name(Alias::new("genre")).to_owned());
            other.values([Alias::new("Roman")]);
        }
    "#;
    assert!(parse_enum_snapshot(source).is_err());
}

#[test]
fn what_creates_no_enum_type_is_not_an_enum_snapshot() {
    for source in [
        "",
        "not rust at all {",
        "fn up() { manager.create_table(Table::create().table(Alias::new(\"books\"))); }",
        // A name without values, values without a name.
        "fn up() { Type::create().as_enum(Alias::new(\"genre\")); }",
        "fn up() { Type::create().values([Alias::new(\"Roman\")]); }",
        // A value that is not written as a name.
        "fn up() { Type::create().as_enum(Alias::new(\"genre\")).values([Alias::new(\"Roman\"), other]); }",
    ] {
        assert!(parse_enum_snapshot(source).is_err(), "{source}");
    }
}

// ── The snapshots of a project ───────────────────────────────────────────────

fn write_snapshot(dir: &TempDir, file: &str, content: &str) {
    let folder = dir.join("snapshots/enums");
    fs::create_dir_all(&folder).unwrap();
    fs::write(folder.join(file), content).unwrap();
}

#[test]
fn a_project_without_the_folder_has_no_enum_snapshot() {
    let dir = TempDir::new();
    assert!(
        read_enum_snapshots(dir.path().to_str().unwrap())
            .unwrap()
            .is_empty()
    );
    // The snapshots of the tables are not enums.
    fs::create_dir_all(dir.join("snapshots")).unwrap();
    fs::write(dir.join("snapshots/books.rs"), "fn up() {}").unwrap();
    assert!(
        read_enum_snapshots(dir.path().to_str().unwrap())
            .unwrap()
            .is_empty()
    );
}

#[test]
fn every_enum_snapshot_of_the_folder_is_read() {
    let dir = TempDir::new();
    let status = ParsedEnum {
        name: "status".into(),
        ..genre(&["Open", "Closed"])
    };
    write_snapshot(&dir, "status.rs", &generate_enum_snapshot_file(&status));
    write_snapshot(
        &dir,
        "genre.rs",
        &generate_enum_snapshot_file(&genre(&["Roman"])),
    );
    // Not a `.rs`: ignored.
    write_snapshot(&dir, "notes.txt", "anything");

    assert_eq!(
        read_enum_snapshots(dir.path().to_str().unwrap()).unwrap(),
        [genre(&["Roman"]), status]
    );
}

#[test]
fn an_unreadable_enum_snapshot_stops_the_reading() {
    let dir = TempDir::new();
    write_snapshot(&dir, "genre.rs", "not a snapshot");
    let error = read_enum_snapshots(dir.path().to_str().unwrap())
        .unwrap_err()
        .to_string();
    let file = dir.join("snapshots/enums/genre.rs").display().to_string();
    assert_eq!(
        error,
        runique_i18n::tf(
            runique_i18n::keys::makemigrations::error::SNAPSHOT_UNREADABLE,
            &[file.as_str()]
        )
    );
}

// A snapshot describes the enum its file is named after, and no other.
#[test]
fn an_enum_snapshot_under_another_name_is_refused() {
    let dir = TempDir::new();
    write_snapshot(
        &dir,
        "mood.rs",
        &generate_enum_snapshot_file(&genre(&["Roman"])),
    );
    let error = read_enum_snapshots(dir.path().to_str().unwrap())
        .unwrap_err()
        .to_string();
    let file = dir.join("snapshots/enums/mood.rs").display().to_string();
    assert_eq!(
        error,
        runique_i18n::tf(
            runique_i18n::keys::makemigrations::error::ENUM_SNAPSHOT_MISNAMED,
            &[file.as_str(), "genre"]
        )
    );
}

// ── What the database has, as the snapshots tell ─────────────────────────────

fn named(name: &str, values: &[&str]) -> ParsedEnum {
    ParsedEnum {
        name: name.into(),
        ..genre(values)
    }
}

/// Two tables: `books.genre` on `Genre`, `orders.status` on `Status`.
fn two_tables() -> [ParsedSchema; 2] {
    [
        table(
            "books",
            vec![enum_column("genre", "Genre", &["Roman", "Policier"])],
        ),
        table("orders", vec![enum_column("status", "Status", &["Open"])]),
    ]
}

// An enum a table uses has the values of its own snapshot, and of no other;
// without a snapshot it has those its column was written with.
#[test]
fn an_enum_has_the_values_of_its_own_snapshot() {
    let snapshots = [named("status", &["Open", "Closed"]), named("other", &["x"])];

    let before = previous_enums(&two_tables(), &snapshots);

    assert_eq!(
        before,
        [
            ParsedEnum {
                places: vec![place("books", "genre")],
                ..genre(&["Roman", "Policier"])
            },
            ParsedEnum {
                places: vec![place("orders", "status")],
                ..named("status", &["Open", "Closed"])
            },
        ]
    );
}

// A snapshot of an enum no table uses is left out: the enum is gone.
#[test]
fn an_enum_no_table_uses_is_not_one_the_database_has() {
    let before = previous_enums(&two_tables(), &[named("mood", &["Calm"])]);
    let names: Vec<&str> = before.iter().map(|known| known.name.as_str()).collect();
    assert_eq!(names, ["genre", "status"]);
    assert!(previous_enums(&[], &[named("mood", &["Calm"])]).is_empty());
}

// The enum columns of a table snapshot take the values of their own enum;
// a column of another enum, or of none, is left as it is.
#[test]
fn a_table_snapshot_takes_the_values_of_each_of_its_enums() {
    let mut schema = table(
        "books",
        vec![
            enum_column("genre", "Genre", &["Roman", "Policier"]),
            enum_column("status", "Status", &["Open"]),
            ParsedColumn {
                name: "title".into(),
                ..ParsedColumn::default()
            },
        ],
    );

    with_enum_values(
        &mut schema,
        &[named("genre", &["Novel", "Policier", "Essai"])],
    );

    assert_eq!(
        schema.columns[0].enum_string_values,
        ["Novel", "Policier", "Essai"]
    );
    assert_eq!(schema.columns[0].enum_name.as_deref(), Some("Genre"));
    assert_eq!(schema.columns[1].enum_string_values, ["Open"]);
    assert!(schema.columns[2].enum_string_values.is_empty());
}

// Only the enums that have no snapshot yet get one written, each in its own
// file, with the values the database has.
#[test]
fn only_the_enums_without_a_snapshot_get_one() {
    let before = [named("genre", &["Roman"]), named("status", &["Open"])];
    let recorded = [named("genre", &["Roman"])];

    let upgrades = enum_snapshot_upgrades(&before, &recorded, "migration/src");

    assert_eq!(upgrades.len(), 1, "{upgrades:?}");
    assert_eq!(upgrades[0].0, "migration/src/snapshots/enums/status.rs");
    assert_eq!(
        parse_enum_snapshot(&upgrades[0].1).unwrap(),
        ParsedEnum {
            places: Vec::new(),
            ..named("status", &["Open"])
        }
    );
    assert!(enum_snapshot_upgrades(&before, &before, "migration/src").is_empty());
    assert_eq!(
        enum_snapshot_upgrades(&before, &[], "migration/src").len(),
        2
    );
}
