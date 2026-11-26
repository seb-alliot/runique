use crate::migration::tests::*;

// ── Date / time / timestamptz column methods ──

const DT_TYPES: &str = r#"
use runique::prelude::*;
model! {
    Sched,
    table: "sched",
    pk: id => Pk,
    {
        d:  date [nullable],
        t:  time [nullable],
        tz: timestamp_tz [nullable],
    }
}
"#;

#[test]
fn date_time_types_map_to_methods() {
    let sql = generate_create_file(&parse_model(DT_TYPES));
    assert!(sql.contains(r#"Alias::new("d")).date()"#), "date:\n{sql}");
    assert!(sql.contains(r#"Alias::new("t")).time()"#), "time:\n{sql}");
    assert!(
        sql.contains(r#"Alias::new("tz")).timestamp_with_time_zone()"#),
        "timestamptz:\n{sql}"
    );
}
