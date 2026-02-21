use crate::runique::migration::{ColumnDef, ModelSchema, PrimaryKeyDef};

#[allow(dead_code)]
pub fn test_all_fields_schema() -> ModelSchema {
    model!("TestAllFields")
        .table_name("test_all_fields")
        .primary_key(PrimaryKeyDef::new("id").uuid())
        // ── Text ──
        .column(ColumnDef::new("f_text").string().nullable())
        .column(ColumnDef::new("f_email").string().nullable())
        .column(ColumnDef::new("f_url").string().nullable())
        .column(ColumnDef::new("f_password").string().nullable())
        .column(ColumnDef::new("f_textarea").text().nullable())
        .column(ColumnDef::new("f_richtext").text().nullable())
        // ── Numeric ──
        .column(ColumnDef::new("f_integer").integer().nullable())
        .column(ColumnDef::new("f_float").string().nullable()) // f32 non supporté → string, à affiner
        .column(ColumnDef::new("f_decimal").string().nullable()) // Decimal non supporté → string, à affiner
        .column(ColumnDef::new("f_percent").integer().nullable())
        .column(ColumnDef::new("f_range").integer().nullable())
        // ── Boolean ──
        .column(ColumnDef::new("f_checkbox").boolean().nullable())
        .column(ColumnDef::new("f_radio_single").boolean().nullable())
        // ── Choice ──
        .column(ColumnDef::new("f_select").string().nullable())
        .column(ColumnDef::new("f_select_multiple").json().nullable())
        .column(ColumnDef::new("f_radio_group").string().nullable())
        .column(ColumnDef::new("f_checkbox_group").json().nullable())
        // ── Datetime ──
        .column(ColumnDef::new("f_date").datetime().nullable()) // Date seul non supporté
        .column(ColumnDef::new("f_time").string().nullable()) // Time non supporté → string
        .column(ColumnDef::new("f_datetime").datetime().nullable())
        .column(ColumnDef::new("f_duration").big_integer().nullable())
        // ── File ──
        .column(ColumnDef::new("f_file_image").json().nullable())
        .column(ColumnDef::new("f_file_document").json().nullable())
        .column(ColumnDef::new("f_file_any").json().nullable())
        // ── Special ──
        .column(ColumnDef::new("f_color").string().nullable())
        .column(ColumnDef::new("f_slug").string().nullable())
        .column(ColumnDef::new("f_uuid").uuid().nullable())
        .column(ColumnDef::new("f_json").json().nullable())
        .column(ColumnDef::new("f_ip").string().nullable())
        // ── Meta ──
        .column(ColumnDef::new("created_at").auto_now())
        .column(ColumnDef::new("updated_at").auto_now_update())
        .build()
        .unwrap()
}
