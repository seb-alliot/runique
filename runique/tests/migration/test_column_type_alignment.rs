//! For every DSL type, the entity `model!{}` generates and its `schema()`
//! declare the very column the migration creates. One mapping
//! (`FieldType::sea_orm_column_type`) feeds all three: SeaORM never guesses a
//! column from the Rust type, and the form engine reads the same column.
//!
//! No database: the migration's column is what the CLI parses, built the way
//! its generated file builds it.
use runique::migration::utils::parser_builder::parse_schema_from_source;
use runique::runique_dsl::ast::FormFieldKind;
use runique::sea_orm::sea_query::{ColumnType, StringLen};

/// One DSL type: a `model!{}` with a single field `v`, the same declaration
/// as text for the CLI's parser, and the column each side declares.
macro_rules! aligned {
    ($module:ident, $kind:ident, [$($attr:tt)*]) => {
        mod $module {
            use runique::prelude::*;

            model! {
                Case,
                table: "align",
                pk: id => i32,
                {
                    v: $kind [$($attr)*],
                }
            }

            pub const SOURCE: &str = concat!(
                "model! { Case, table: \"align\", pk: id => i32, { v: ",
                stringify!($kind),
                " [",
                stringify!($($attr)*),
                "], } }"
            );

            pub fn entity() -> runique::sea_orm::sea_query::ColumnType {
                use runique::sea_orm::ColumnTrait;
                Column::V.def().get_column_type().clone()
            }

            /// Whether the field the generated admin form gives `v` can
            /// hold the entity's column: what the admin checks at boot.
            pub fn form_accepts() -> bool {
                use runique::forms::{Forms, RuniqueForm};
                let mut form = Forms::new("");
                AdminForm::register_fields(&mut form);
                form.fields
                    .get("v")
                    .expect("`v` in the generated admin form")
                    .accepts_column(&entity())
            }

            pub fn schema_type() -> runique::sea_orm::sea_query::ColumnType {
                schema()
                    .columns
                    .into_iter()
                    .find(|c| c.name == "v")
                    .expect("`v` in the schema")
                    .col_type
            }
        }
    };
}

aligned!(t_text, text, [required]);
aligned!(t_text_len, text, [required, max_length: 80]);
aligned!(t_email, email, [required]);
aligned!(t_password, password, [required]);
aligned!(t_richtext, richtext, [required]);
aligned!(t_textarea, textarea, [required]);
aligned!(t_url, url, [required]);
aligned!(t_int, int, [required]);
aligned!(t_float, float, [required]);
aligned!(t_decimal, decimal, [required, max_digits: 10, decimal_places: 2]);
aligned!(t_percent, percent, [required]);
aligned!(t_bool, bool, [required]);
aligned!(t_date, date, [required]);
aligned!(t_time, time, [required]);
aligned!(t_datetime, datetime, [required]);
aligned!(t_image, image, [required, upload_to: "media"]);
aligned!(t_document, document, [required, upload_to: "media"]);
aligned!(t_file, file, [required, upload_to: "media"]);
aligned!(t_color, color, [required]);
aligned!(t_slug, slug, [required]);
aligned!(t_uuid, uuid, [required]);
aligned!(t_json, json, [required]);
aligned!(t_ip, ip, [required]);
aligned!(t_bigint, bigint, [required]);
aligned!(t_phone, phone, [required]);
aligned!(t_char, char, [required]);
aligned!(t_i8, i8, [required]);
aligned!(t_i16, i16, [required]);
aligned!(t_u32, u32, [required]);
aligned!(t_u64, u64, [required]);
aligned!(t_f32, f32, [required]);
aligned!(t_timestamp, timestamp, [required]);
aligned!(t_timestamp_tz, timestamp_tz, [required]);
aligned!(t_json_binary, json_binary, [required]);
aligned!(t_binary, binary, [required]);
aligned!(t_binary_len, binary, [required, max_length: 16]);
aligned!(t_var_binary, var_binary, [required]);
aligned!(t_blob, blob, [required]);
aligned!(t_cidr, cidr, [required]);
aligned!(t_mac_address, mac_address, [required]);
aligned!(t_interval, interval, [required]);

type Case = (
    &'static str,
    fn() -> ColumnType,
    fn() -> ColumnType,
    fn() -> bool,
);

/// Every DSL type, with no catch-all: a new type doesn't compile here until
/// it says which column it stores, or why it has none.
fn cases_of(kind: FormFieldKind) -> Vec<Case> {
    use FormFieldKind::*;
    macro_rules! of {
        ($($module:ident),+) => {
            vec![$(($module::SOURCE, $module::entity as fn() -> ColumnType, $module::schema_type as fn() -> ColumnType, $module::form_accepts as fn() -> bool)),+]
        };
    }
    match kind {
        Text => of!(t_text, t_text_len),
        Email => of!(t_email),
        Password => of!(t_password),
        Richtext => of!(t_richtext),
        Textarea => of!(t_textarea),
        Url => of!(t_url),
        Int => of!(t_int),
        Float => of!(t_float),
        Decimal => of!(t_decimal),
        Percent => of!(t_percent),
        Bool => of!(t_bool),
        Date => of!(t_date),
        Time => of!(t_time),
        Datetime => of!(t_datetime),
        Image => of!(t_image),
        Document => of!(t_document),
        File => of!(t_file),
        Color => of!(t_color),
        Slug => of!(t_slug),
        Uuid => of!(t_uuid),
        Json => of!(t_json),
        Ip => of!(t_ip),
        Bigint => of!(t_bigint),
        Phone => of!(t_phone),
        Char => of!(t_char),
        I8 => of!(t_i8),
        I16 => of!(t_i16),
        U32 => of!(t_u32),
        U64 => of!(t_u64),
        F32 => of!(t_f32),
        Timestamp => of!(t_timestamp),
        TimestampTz => of!(t_timestamp_tz),
        JsonBinary => of!(t_json_binary),
        Binary => of!(t_binary, t_binary_len),
        VarBinary => of!(t_var_binary),
        Blob => of!(t_blob),
        Cidr => of!(t_cidr),
        MacAddress => of!(t_mac_address),
        Interval => of!(t_interval),
        // An enum's column is typed by its own `DeriveActiveEnum`.
        Choice | Radio => vec![],
        // A list lives in a table of its own, not in a column.
        Checkbox | Multichoice => vec![],
    }
}

/// The column the CLI creates for this declaration, as its generated
/// migration builds it (`col_type_method`).
fn migration_column_type(source: &str) -> ColumnType {
    let (_, schema) = parse_schema_from_source(source)
        .unwrap()
        .expect("the CLI parses the model");
    let column = &schema.columns[0];
    match column.col_type.as_str() {
        "String" => ColumnType::String(column.max_length.map_or(StringLen::None, StringLen::N)),
        "Text" => ColumnType::Text,
        "TinyInteger" => ColumnType::TinyInteger,
        "SmallInteger" => ColumnType::SmallInteger,
        "Integer" => ColumnType::Integer,
        "BigInteger" => ColumnType::BigInteger,
        "Unsigned" => ColumnType::Unsigned,
        "BigUnsigned" => ColumnType::BigUnsigned,
        "Float" => ColumnType::Float,
        "Double" => ColumnType::Double,
        "Decimal" => ColumnType::Decimal(column.decimal_len),
        "Boolean" => ColumnType::Boolean,
        "Date" => ColumnType::Date,
        "Time" => ColumnType::Time,
        "DateTime" => ColumnType::DateTime,
        "TimestampWithTimeZone" => ColumnType::TimestampWithTimeZone,
        "Uuid" => ColumnType::Uuid,
        "Json" => ColumnType::Json,
        "Binary" => ColumnType::Binary(column.max_length.unwrap_or(1)),
        "VarBinary" => ColumnType::VarBinary(StringLen::N(column.max_length.unwrap_or(255))),
        "Blob" => ColumnType::Blob,
        other => panic!("the migration writes `{other}`, which this test doesn't know"),
    }
}

#[test]
fn every_type_declares_the_column_the_migration_creates() {
    let mut checked = 0;
    for &kind in FormFieldKind::ALL {
        for (source, entity, schema, _) in cases_of(kind) {
            let migration = migration_column_type(source);
            assert_eq!(
                entity(),
                migration,
                "{kind:?}: the entity and the migration disagree"
            );
            assert_eq!(
                schema(),
                migration,
                "{kind:?}: `schema()` and the migration disagree"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 41, "every declared case was checked");
}

#[test]
fn every_generated_form_field_holds_its_column() {
    let mut checked = 0;
    for &kind in FormFieldKind::ALL {
        for (source, _, _, form_accepts) in cases_of(kind) {
            assert!(
                form_accepts(),
                "{kind:?}: the generated form field can't hold the entity's column ({source})"
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 41, "a case was dropped");
}
