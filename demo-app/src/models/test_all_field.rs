use runique::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "test_all_fields")]
#[allow(dead_code)]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,

    // ── Text ──
    pub f_text: Option<String>,
    pub f_email: Option<String>,
    pub f_url: Option<String>,
    pub f_password: Option<String>,
    pub f_textarea: Option<String>,
    pub f_richtext: Option<String>,

    // readonly / disabled : pas stockés (logique UI)

    // ── Numeric ──
    pub f_integer: Option<i32>,
    pub f_float: Option<f32>,
    pub f_decimal: Option<Decimal>,
    pub f_percent: Option<i32>,
    pub f_range: Option<i32>,

    // ── Boolean ──
    pub f_checkbox: Option<bool>,
    pub f_radio_single: Option<bool>,

    // ── Choice ──
    pub f_select: Option<String>,
    pub f_select_multiple: Option<Value>, // JSON array
    pub f_radio_group: Option<String>,
    pub f_checkbox_group: Option<Value>, // JSON array

    // ── Datetime ──
    pub f_date: Option<Date>,
    pub f_time: Option<Time>,
    pub f_datetime: Option<DateTime<Utc>>,
    pub f_duration: Option<i64>,

    // ── File ──
    pub f_file_image: Option<Value>, // { path, size, mime }
    pub f_file_document: Option<Value>,
    pub f_file_any: Option<Value>, // array

    // ── Special ──
    pub f_color: Option<String>,
    pub f_slug: Option<String>,
    pub f_uuid: Option<Uuid>,
    pub f_json: Option<Value>,
    pub f_ip: Option<String>,

    // ── Meta ──
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
#[allow(dead_code)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl_objects!(Entity);
