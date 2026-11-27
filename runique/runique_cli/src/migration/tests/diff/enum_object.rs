use crate::migration::tests::*;

fn genre(values: &[&str]) -> ParsedEnum {
    ParsedEnum {
        name: "genre".into(),
        values: values.iter().map(|value| value.to_string()).collect(),
        places: vec![EnumPlace {
            table: "books".into(),
            column: "genre".into(),
        }],
    }
}

fn pairs(renamed: &[(&str, &str)]) -> Vec<(String, String)> {
    renamed
        .iter()
        .map(|(old, new)| (old.to_string(), new.to_string()))
        .collect()
}

// ── An enum against its snapshot ─────────────────────────────────────────────

#[test]
fn an_enum_without_a_snapshot_is_created_with_its_values() {
    let declared = genre(&["Roman", "Policier"]);
    let changes = diff_enum(None, &declared);

    assert_eq!(
        changes,
        EnumChanges {
            name: "genre".into(),
            change: EnumChange::Created,
            values: declared.values.clone(),
            places: declared.places.clone(),
            ..EnumChanges::default()
        }
    );
    assert!(!changes.is_empty());
}

#[test]
fn an_enum_as_its_snapshot_has_it_has_nothing_to_migrate() {
    let declared = genre(&["Roman", "Policier"]);
    let changes = diff_enum(Some(&declared), &declared);

    assert_eq!(changes.change, EnumChange::Altered);
    assert!(changes.is_empty(), "{changes:?}");
}

// The places are not compared: a snapshot holds none.
#[test]
fn the_places_alone_are_not_a_change() {
    let snapshot = ParsedEnum {
        places: Vec::new(),
        ..genre(&["Roman"])
    };
    assert!(diff_enum(Some(&snapshot), &genre(&["Roman"])).is_empty());
}

// Same position, another value, the former one gone and the new one fresh:
// one rename, and neither an addition nor a removal.
#[test]
fn a_value_changed_in_place_is_renamed() {
    let changes = diff_enum(
        Some(&genre(&["Roman", "Policier"])),
        &genre(&["Novel", "Policier"]),
    );

    assert_eq!(changes.renamed_values, pairs(&[("Roman", "Novel")]));
    assert!(changes.added_values.is_empty() && changes.dropped_values.is_empty());
    assert_eq!(changes.values, ["Novel", "Policier"]);
    assert!(!changes.is_empty());
}

#[test]
fn several_values_are_renamed_each_at_its_place() {
    let changes = diff_enum(Some(&genre(&["a", "b", "c"])), &genre(&["x", "b", "z"]));
    assert_eq!(changes.renamed_values, pairs(&[("a", "x"), ("c", "z")]));
    assert!(changes.added_values.is_empty() && changes.dropped_values.is_empty());
}

// Added values come out in the order the model declares them.
#[test]
fn values_added_are_given_in_the_order_of_the_model() {
    let changes = diff_enum(
        Some(&genre(&["Roman"])),
        &genre(&["Roman", "Zeta", "Alpha", "Mid"]),
    );

    assert_eq!(changes.added_values, ["Zeta", "Alpha", "Mid"]);
    assert!(changes.renamed_values.is_empty() && changes.dropped_values.is_empty());
}

#[test]
fn values_taken_out_are_given_in_the_order_of_the_snapshot() {
    let changes = diff_enum(Some(&genre(&["c", "a", "b", "d"])), &genre(&["a"]));

    // `c` is at the place `a` now holds, but `a` is not new: no rename.
    assert_eq!(changes.dropped_values, ["c", "b", "d"]);
    assert!(changes.renamed_values.is_empty() && changes.added_values.is_empty());
}

// Two values that swap places are still the same two values.
#[test]
fn values_put_in_another_order_are_no_change() {
    let changes = diff_enum(Some(&genre(&["a", "b"])), &genre(&["b", "a"]));
    assert!(changes.is_empty(), "{changes:?}");
}

// The former value is still there, further on: the new one is an addition.
#[test]
fn a_value_inserted_before_another_is_added_not_renamed() {
    let changes = diff_enum(Some(&genre(&["a", "b"])), &genre(&["new", "a", "b"]));

    assert_eq!(changes.added_values, ["new"]);
    assert!(changes.renamed_values.is_empty() && changes.dropped_values.is_empty());
}

// A rename, an addition and a removal in one go: each is found as what it is.
#[test]
fn a_rename_an_addition_and_a_removal_are_told_apart() {
    let changes = diff_enum(
        Some(&genre(&["Roman", "Policier", "Essai", "Conte"])),
        &genre(&["Novel", "Policier", "Conte", "Drama"]),
    );

    // `Roman` gave its place to `Novel`. `Essai` is gone, but its place is
    // held by `Conte`, which is not new: taken out, not renamed.
    assert_eq!(changes.renamed_values, pairs(&[("Roman", "Novel")]));
    assert_eq!(changes.added_values, ["Drama"]);
    assert_eq!(changes.dropped_values, ["Essai"]);
}

// ── The enums of a project against its snapshots ─────────────────────────────

fn named(name: &str, values: &[&str]) -> ParsedEnum {
    ParsedEnum {
        name: name.into(),
        ..genre(values)
    }
}

// Only what has something to migrate comes out, in the order of the models.
#[test]
fn only_the_enums_with_something_to_migrate_come_out() {
    let snapshots = [named("genre", &["Roman"]), named("status", &["Open"])];
    let declared = [
        named("genre", &["Roman"]),
        named("mood", &["Calm"]),
        named("status", &["Open", "Closed"]),
    ];

    let changes = diff_enums(&snapshots, &declared);

    let found: Vec<(&str, EnumChange)> = changes
        .iter()
        .map(|change| (change.name.as_str(), change.change))
        .collect();
    assert_eq!(
        found,
        [
            ("mood", EnumChange::Created),
            ("status", EnumChange::Altered)
        ]
    );
    assert_eq!(changes[1].added_values, ["Closed"]);
}

// An enum is matched to its snapshot by its whole name: a snapshot whose
// name only starts like it, and whose column it doesn't use, is another.
#[test]
fn an_enum_is_matched_to_the_snapshot_of_its_own_name() {
    let elsewhere = ParsedEnum {
        places: vec![EnumPlace {
            table: "shops".into(),
            column: "genres".into(),
        }],
        ..named("genres", &["Roman"])
    };
    let changes = diff_enums(&[elsewhere], &[named("genre", &["Roman"])]);
    assert_eq!(changes.len(), 1);
    assert_eq!(changes[0].change, EnumChange::Created);
}

// An enum no model declares any more is not in the result.
#[test]
fn an_enum_the_models_no_longer_declare_is_left_out() {
    let snapshots = [named("genre", &["Roman"])];
    assert!(diff_enums(&snapshots, &[]).is_empty());
    assert!(diff_enums(&[], &[]).is_empty());
}

// ── An enum renamed ──────────────────────────────────────────────────────────

/// An enum under `name`, used by `books.<column>` for each of `columns`.
fn used_by(name: &str, values: &[&str], columns: &[&str]) -> ParsedEnum {
    ParsedEnum {
        name: name.into(),
        values: values.iter().map(|value| value.to_string()).collect(),
        places: columns
            .iter()
            .map(|column| EnumPlace {
                table: "books".into(),
                column: column.to_string(),
            })
            .collect(),
    }
}

fn summary(changes: &[EnumChanges]) -> Vec<(&str, EnumChange, Option<&str>)> {
    changes
        .iter()
        .map(|change| {
            (
                change.name.as_str(),
                change.change,
                change.renamed_from.as_deref(),
            )
        })
        .collect()
}

// The enum the database has is no longer declared, and its column now uses
// one the database doesn't have: the same enum under another name.
#[test]
fn an_enum_whose_column_now_uses_a_new_one_is_renamed() {
    let before = [used_by("genre", &["Roman", "Policier"], &["genre"])];
    let declared = [used_by("category", &["Roman", "Policier"], &["genre"])];

    let changes = diff_enums(&before, &declared);

    // Renamed, and nothing else: its values are the ones it had.
    assert_eq!(
        summary(&changes),
        [("category", EnumChange::Renamed, Some("genre"))]
    );
    assert_eq!(changes[0].values, ["Roman", "Policier"]);
    assert_eq!(changes[0].places, declared[0].places);
    assert!(!changes[0].is_empty());
}

// Renamed, and a value renamed in the same run: two entries, the rename of
// the enum first, then its values compared with the ones it had.
#[test]
fn a_renamed_enum_has_its_values_compared_with_the_former_ones() {
    let before = [used_by("genre", &["Roman", "Policier"], &["genre"])];
    let declared = [used_by(
        "category",
        &["Novel", "Policier", "Essai"],
        &["genre"],
    )];

    let changes = diff_enums(&before, &declared);

    assert_eq!(
        summary(&changes),
        [
            ("category", EnumChange::Renamed, Some("genre")),
            ("category", EnumChange::Altered, None),
        ]
    );
    assert_eq!(changes[1].renamed_values, pairs(&[("Roman", "Novel")]));
    assert_eq!(changes[1].added_values, ["Essai"]);
    assert!(changes[1].dropped_values.is_empty());
}

// The former enum is still declared: the new one is a new one.
#[test]
fn an_enum_is_not_renamed_while_the_former_one_is_still_declared() {
    let before = [used_by("genre", &["Roman"], &["genre", "former"])];
    let declared = [
        used_by("genre", &["Roman"], &["former"]),
        used_by("category", &["Roman"], &["genre"]),
    ];

    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [("category", EnumChange::Created, None)]
    );
}

// No column of the former enum uses the new one: nothing ties them, even
// with the same values.
#[test]
fn an_enum_is_not_renamed_without_a_column_in_common() {
    let before = [used_by("genre", &["Roman"], &["genre"])];
    let declared = [used_by("category", &["Roman"], &["category"])];

    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [("category", EnumChange::Created, None)]
    );
}

// The new enum already exists in the database: its values are compared,
// whatever the columns it took from another.
#[test]
fn an_enum_the_database_has_is_never_the_heir_of_another() {
    let before = [
        used_by("genre", &["Roman"], &["genre"]),
        used_by("category", &["Roman"], &["category"]),
    ];
    let declared = [used_by(
        "category",
        &["Roman", "Essai"],
        &["category", "genre"],
    )];

    let changes = diff_enums(&before, &declared);
    assert_eq!(summary(&changes), [("category", EnumChange::Altered, None)]);
    assert_eq!(changes[0].added_values, ["Essai"]);
}

// A former enum is taken once: the first new enum that shares a column with
// it is the renamed one, the next is created.
#[test]
fn a_former_enum_is_taken_once() {
    let before = [used_by("genre", &["Roman"], &["genre", "former"])];
    let declared = [
        used_by("category", &["Roman"], &["genre"]),
        used_by("mood", &["Roman"], &["former"]),
    ];

    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [
            ("category", EnumChange::Renamed, Some("genre")),
            ("mood", EnumChange::Created, None),
        ]
    );
}

// Two former enums, two new ones: each new one takes the former enum whose
// column it has, not the first that is free.
#[test]
fn each_renamed_enum_follows_its_own_column() {
    let before = [
        used_by("genre", &["Roman"], &["genre"]),
        used_by("status", &["Open"], &["status"]),
    ];
    let declared = [
        used_by("category", &["Roman"], &["genre"]),
        used_by("state", &["Open"], &["status"]),
    ];

    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [
            ("category", EnumChange::Renamed, Some("genre")),
            ("state", EnumChange::Renamed, Some("status")),
        ]
    );
    // The other way round in the names: `state` comes before the enum that
    // takes `genre`, and still takes `status`.
    let declared = [
        used_by("alpha", &["Open"], &["status"]),
        used_by("category", &["Roman"], &["genre"]),
    ];
    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [
            ("alpha", EnumChange::Renamed, Some("status")),
            ("category", EnumChange::Renamed, Some("genre")),
        ]
    );
}

// More columns on the renamed enum than the former one had: still renamed.
#[test]
fn a_renamed_enum_may_be_used_by_more_columns() {
    let before = [used_by("genre", &["Roman"], &["genre"])];
    let declared = [used_by("category", &["Roman"], &["genre", "second"])];

    assert_eq!(
        summary(&diff_enums(&before, &declared)),
        [("category", EnumChange::Renamed, Some("genre"))]
    );
}
