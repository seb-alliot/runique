//! A field's cleaned text, read into the value its column stores. One reading
//! for every field type, the same `admin_from_form` makes from the model.
use crate::forms::base::FieldConfig;
use crate::forms::fields::{decode_binary, parse_datetime_local, parse_utc_datetime};
use crate::utils::trad::t;
use sea_orm::Value;
use sea_orm::sea_query::ColumnType;

/// A field's value read as its column, or as `default` — what the field type
/// stores when no column was given to it. `Ok(None)` when empty.
pub(crate) fn read_field(
    config: &FieldConfig,
    raw: &str,
    default: ColumnType,
) -> Result<Option<Value>, String> {
    if raw.is_empty() {
        return Ok(None);
    }
    column_value(config.column_type.as_ref().unwrap_or(&default), raw).map(Some)
}

/// `raw` as `column` stores it. `Err` when the text doesn't read as that
/// column's type, or the column is one no form field writes.
pub fn column_value(column: &ColumnType, raw: &str) -> Result<Value, String> {
    let unreadable = || t("forms.value_unconvertible").into_owned();
    let number = raw.replace(',', ".");
    let value = match column {
        ColumnType::String(_) | ColumnType::Text | ColumnType::Char(_) => {
            Value::from(raw.to_string())
        }
        // An enum column takes the variant's stored text: the entity's
        // `DeriveActiveEnum` reads it back, refusing an unknown one.
        ColumnType::Enum { .. } => Value::from(raw.to_string()),
        ColumnType::TinyInteger => Value::from(raw.parse::<i8>().map_err(|_| unreadable())?),
        ColumnType::SmallInteger => Value::from(raw.parse::<i16>().map_err(|_| unreadable())?),
        ColumnType::Integer => Value::from(raw.parse::<i32>().map_err(|_| unreadable())?),
        ColumnType::BigInteger => Value::from(raw.parse::<i64>().map_err(|_| unreadable())?),
        ColumnType::TinyUnsigned => Value::from(raw.parse::<u8>().map_err(|_| unreadable())?),
        ColumnType::SmallUnsigned => Value::from(raw.parse::<u16>().map_err(|_| unreadable())?),
        ColumnType::Unsigned => Value::from(raw.parse::<u32>().map_err(|_| unreadable())?),
        ColumnType::BigUnsigned => Value::from(raw.parse::<u64>().map_err(|_| unreadable())?),
        ColumnType::Float => Value::from(
            number
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(unreadable)?,
        ),
        ColumnType::Double => Value::from(
            number
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(unreadable)?,
        ),
        ColumnType::Decimal(_) => {
            Value::from(rust_decimal::Decimal::from_str_exact(&number).map_err(|_| unreadable())?)
        }
        ColumnType::Boolean => Value::from(matches!(
            raw.to_ascii_lowercase().as_str(),
            "true" | "1" | "on"
        )),
        ColumnType::Date => Value::from(
            chrono::NaiveDate::parse_from_str(raw, "%Y-%m-%d").map_err(|_| unreadable())?,
        ),
        ColumnType::Time => Value::from(
            chrono::NaiveTime::parse_from_str(raw, "%H:%M:%S")
                .or_else(|_| chrono::NaiveTime::parse_from_str(raw, "%H:%M"))
                .map_err(|_| unreadable())?,
        ),
        ColumnType::DateTime | ColumnType::Timestamp => {
            Value::from(parse_datetime_local(raw).ok_or_else(unreadable)?)
        }
        ColumnType::TimestampWithTimeZone => {
            Value::from(parse_utc_datetime(raw).ok_or_else(unreadable)?)
        }
        ColumnType::Uuid => Value::from(uuid::Uuid::parse_str(raw).map_err(|_| unreadable())?),
        ColumnType::Json | ColumnType::JsonBinary => {
            Value::from(serde_json::from_str::<serde_json::Value>(raw).map_err(|_| unreadable())?)
        }
        ColumnType::Binary(_) | ColumnType::VarBinary(_) | ColumnType::Blob => {
            Value::from(decode_binary(raw).ok_or_else(unreadable)?)
        }
        // Native Postgres types, money, arrays, vectors…: the migration never
        // creates them from the DSL, and no field knows how to fill one.
        _ => return Err(unreadable()),
    };
    Ok(value)
}
