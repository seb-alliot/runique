//! What `model!{}` makes of a field's attributes: the admin form's limits,
//! rows, label and upload settings, the schema's column types and meta, and
//! the enums it declares.
use crate::utils::env::{del_env, set_env};
use runique::forms::{CommonFieldConfig, Forms};
use runique::forms::field::RuniqueForm;
use runique::migration::OrderDir;
use runique::sea_orm::ActiveEnum;
use runique::sea_orm::sea_query::ColumnType;
use serial_test::serial;

mod attrs {
    use runique::prelude::*;

    model! {
        Attr,
        table: "dsl_attrs",
        pk: id => i32,
        enums: {
            Level: i32 [Low = 1, High = 2],
            Mode: [Plain, Fancy: "Décoré"],
        },
        {
            title: text [required, min_length: 3, max_length: 20, label: "Titre"],
            body: textarea [nullable, rows: 7],
            data: json [nullable, rows: 5],
            qty: int [nullable, min: 0, max: 10],
            ratio: float [nullable, min: 0.5, max: 9.5],
            scan: file [nullable, upload_to: "scans/", max_size: 2KB],
            level: choice [nullable, enum(Level)],
            mode: choice [nullable, enum(Mode)],
            code: text [nullable, unique],
        },
        meta: {
            ordering: [-title],
            unique_together: [(title, code)],
        }
    }
}

fn admin_form() -> Forms {
    let mut form = Forms::new("csrf");
    attrs::AttrAdminForm::register_fields(&mut form);
    form
}

fn rows(form: &Forms, name: &str) -> Option<u64> {
    form.fields[name]
        .get_field_config()
        .extra_context
        .get("rows")?
        .as_u64()
}

#[test]
fn the_admin_form_keeps_the_declared_label_and_limits() {
    let form = admin_form();
    assert_eq!(form.fields["title"].label(), "Titre");

    let title = form.fields["title"].bounds();
    assert_eq!((title.min_length, title.max_length), (Some(3), Some(20)));
    assert_eq!(form.fields["qty"].bounds().max_int, Some(10));
    let ratio = form.fields["ratio"].bounds();
    assert_eq!((ratio.min_float, ratio.max_float), (Some(0.5), Some(9.5)));
}

#[test]
fn a_textarea_and_a_json_field_keep_their_rows() {
    let form = admin_form();
    assert_eq!(rows(&form, "body"), Some(7));
    assert_eq!(rows(&form, "data"), Some(5));
}

#[test]
fn a_file_field_keeps_its_size_limit() {
    assert_eq!(admin_form().fields["scan"].model_max_size(), Some(2048));
}

#[tokio::test]
#[serial]
async fn a_file_field_commits_under_its_upload_to() {
    let media = std::env::temp_dir().join(format!("rq_attrs_{}", uuid::Uuid::new_v4()));
    let staging = media.join(".staging-attrs");
    std::fs::create_dir_all(&staging).unwrap();
    let staged = staging.join("doc.txt");
    std::fs::write(&staged, "x").unwrap();
    set_env("MEDIA_ROOT", media.to_str().unwrap());

    let mut form = admin_form();
    let field = form.fields.get_mut("scan").unwrap();
    field.set_submitted_value(staged.to_str().unwrap());
    let finalized = field.finalize().await;
    let value = field.value().to_string();

    del_env("MEDIA_ROOT");
    let _ = std::fs::remove_dir_all(&media);
    finalized.expect("finalize");
    assert!(value.starts_with("scans/"), "{value}");
}

#[test]
fn the_schema_keeps_column_types_and_meta() {
    let schema = attrs::schema();
    let col = |name: &str| {
        schema
            .columns
            .iter()
            .find(|c| c.name == name)
            .unwrap_or_else(|| panic!("column {name}"))
            .col_type
            .clone()
    };
    assert_eq!(col("qty"), ColumnType::Integer);
    // The enum is looked up by name: `Mode` is stored as text, `Level` as i32.
    assert_eq!(col("level"), ColumnType::Integer);

    assert!(matches!(
        schema.ordering.as_slice(),
        [(field, OrderDir::Desc)] if field == "title"
    ));
    assert_eq!(
        schema.unique_together,
        vec![vec!["title".to_string(), "code".to_string()]]
    );
    assert_eq!(attrs::UNIQUE_FIELDS, ["code"]);
}

#[test]
fn an_i32_enum_is_an_integer_column() {
    assert_eq!(
        attrs::Level::db_type().get_column_type(),
        &ColumnType::Integer
    );
}

#[test]
fn an_enum_is_read_back_from_its_label_and_shown_with_it() {
    assert_eq!("Décoré".parse::<attrs::Mode>(), Ok(attrs::Mode::Fancy));
    assert_eq!("Fancy".parse::<attrs::Mode>(), Ok(attrs::Mode::Fancy));

    let mut row = runique::serde_json::json!({ "mode": "Fancy", "title": "Fancy" });
    attrs::apply_enum_labels(&mut row);
    assert_eq!(row["mode"], "Décoré");
    assert_eq!(row["title"], "Fancy", "only the enum column is relabelled");
}
