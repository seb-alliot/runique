use crate::migration::tests::*;

// ── Reading a file back: the columns in the order they were written ──

// A snapshot and a creation are read back with their columns in the order of
// the file, which is the order of the model: a table created again from its
// snapshot must not come back with its columns reversed.
#[test]
fn columns_are_read_back_in_the_order_of_the_file() {
    let column = |name: &str| ParsedColumn {
        name: name.into(),
        col_type: crate::migration::ColumnType::String,
        ..ParsedColumn::default()
    };
    let schema = ParsedSchema {
        table_name: "t".into(),
        primary_key: Some(ParsedColumn {
            name: "id".into(),
            col_type: crate::migration::ColumnType::Integer,
            ..ParsedColumn::default()
        }),
        columns: vec![
            column("zeta"),
            column("alpha"),
            column("mid"),
            column("beta"),
        ],
        foreign_keys: vec![],
        indexes: vec![],
    };
    let names = |source: String| -> Vec<String> {
        parse_seaorm_source(&source)
            .unwrap()
            .columns
            .into_iter()
            .map(|c| c.name)
            .collect()
    };

    assert_eq!(
        names(generate_snapshot_file(&schema)),
        ["zeta", "alpha", "mid", "beta"]
    );
    assert_eq!(
        names(generate_create_file(&schema)),
        ["zeta", "alpha", "mid", "beta"]
    );
}
