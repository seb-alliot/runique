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

/// The six spellings a checkbox or a form sends, in any case. Anything else
/// isn't a boolean: a forged `banana` is refused, never read as `false`.
pub fn parse_bool(raw: &str) -> Option<bool> {
    match raw.to_ascii_lowercase().as_str() {
        "true" | "1" | "on" => Some(true),
        "false" | "0" | "off" => Some(false),
        _ => None,
    }
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
        ColumnType::Boolean => Value::from(parse_bool(raw).ok_or_else(unreadable)?),
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

/// `column`'s NULL, typed as its values are: what an emptied nullable field
/// writes. `None` for a column no field fills.
pub(crate) fn null_value(column: &ColumnType) -> Option<Value> {
    Some(match column {
        ColumnType::String(_)
        | ColumnType::Text
        | ColumnType::Char(_)
        | ColumnType::Enum { .. } => Value::from(None::<String>),
        ColumnType::TinyInteger => Value::from(None::<i8>),
        ColumnType::SmallInteger => Value::from(None::<i16>),
        ColumnType::Integer => Value::from(None::<i32>),
        ColumnType::BigInteger => Value::from(None::<i64>),
        ColumnType::TinyUnsigned => Value::from(None::<u8>),
        ColumnType::SmallUnsigned => Value::from(None::<u16>),
        ColumnType::Unsigned => Value::from(None::<u32>),
        ColumnType::BigUnsigned => Value::from(None::<u64>),
        ColumnType::Float => Value::from(None::<f32>),
        ColumnType::Double => Value::from(None::<f64>),
        ColumnType::Decimal(_) => Value::from(None::<rust_decimal::Decimal>),
        ColumnType::Boolean => Value::from(None::<bool>),
        ColumnType::Date => Value::from(None::<chrono::NaiveDate>),
        ColumnType::Time => Value::from(None::<chrono::NaiveTime>),
        ColumnType::DateTime | ColumnType::Timestamp => Value::from(None::<chrono::NaiveDateTime>),
        ColumnType::TimestampWithTimeZone => Value::from(None::<chrono::DateTime<chrono::Utc>>),
        ColumnType::Uuid => Value::from(None::<uuid::Uuid>),
        ColumnType::Json | ColumnType::JsonBinary => Value::from(None::<serde_json::Value>),
        ColumnType::Binary(_) | ColumnType::VarBinary(_) | ColumnType::Blob => {
            Value::from(None::<Vec<u8>>)
        }
        _ => return None,
    })
}

#[cfg(test)]
mod null_value_tests {
    use super::*;
    use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
    use sea_orm::sea_query::{Alias, IntoIden, StringLen};

    #[test]
    fn text_like_columns_give_a_string_null() {
        let enum_column = ColumnType::Enum {
            name: Alias::new("e").into_iden(),
            variants: Vec::new(),
        };
        for column in [
            ColumnType::String(StringLen::None),
            ColumnType::Text,
            ColumnType::Char(None),
            enum_column,
        ] {
            assert_eq!(null_value(&column), Some(Value::String(None)));
        }
    }

    #[test]
    fn each_integer_width_gives_its_own_null() {
        assert_eq!(
            null_value(&ColumnType::TinyInteger),
            Some(Value::TinyInt(None))
        );
        assert_eq!(
            null_value(&ColumnType::SmallInteger),
            Some(Value::SmallInt(None))
        );
        assert_eq!(null_value(&ColumnType::Integer), Some(Value::Int(None)));
        assert_eq!(
            null_value(&ColumnType::BigInteger),
            Some(Value::BigInt(None))
        );
        assert_eq!(
            null_value(&ColumnType::TinyUnsigned),
            Some(Value::TinyUnsigned(None))
        );
        assert_eq!(
            null_value(&ColumnType::SmallUnsigned),
            Some(Value::SmallUnsigned(None))
        );
        assert_eq!(
            null_value(&ColumnType::Unsigned),
            Some(Value::Unsigned(None))
        );
        assert_eq!(
            null_value(&ColumnType::BigUnsigned),
            Some(Value::BigUnsigned(None))
        );
    }

    #[test]
    fn floats_booleans_and_decimals_give_their_own_null() {
        assert_eq!(null_value(&ColumnType::Float), Some(Value::Float(None)));
        assert_eq!(null_value(&ColumnType::Double), Some(Value::Double(None)));
        assert_eq!(null_value(&ColumnType::Boolean), Some(Value::Bool(None)));
        assert_eq!(
            null_value(&ColumnType::Decimal(None)),
            Some(Value::from(None::<rust_decimal::Decimal>))
        );
    }

    #[test]
    fn date_and_time_columns_give_their_own_null() {
        assert_eq!(
            null_value(&ColumnType::Date),
            Some(Value::from(None::<NaiveDate>))
        );
        assert_eq!(
            null_value(&ColumnType::Time),
            Some(Value::from(None::<NaiveTime>))
        );
        for column in [ColumnType::DateTime, ColumnType::Timestamp] {
            assert_eq!(
                null_value(&column),
                Some(Value::from(None::<NaiveDateTime>))
            );
        }
        assert_eq!(
            null_value(&ColumnType::TimestampWithTimeZone),
            Some(Value::from(None::<DateTime<Utc>>))
        );
    }

    #[test]
    fn uuid_json_and_binary_columns_give_their_own_null() {
        assert_eq!(
            null_value(&ColumnType::Uuid),
            Some(Value::from(None::<uuid::Uuid>))
        );
        for column in [ColumnType::Json, ColumnType::JsonBinary] {
            assert_eq!(
                null_value(&column),
                Some(Value::from(None::<serde_json::Value>))
            );
        }
        for column in [
            ColumnType::Binary(4),
            ColumnType::VarBinary(StringLen::None),
            ColumnType::Blob,
        ] {
            assert_eq!(null_value(&column), Some(Value::from(None::<Vec<u8>>)));
        }
    }

    #[test]
    fn a_column_no_field_fills_has_no_null() {
        assert_eq!(null_value(&ColumnType::Money(None)), None);
        assert_eq!(null_value(&ColumnType::Inet), None);
    }

    #[test]
    fn a_null_is_a_different_variant_from_a_present_value() {
        assert_ne!(null_value(&ColumnType::Integer), Some(Value::BigInt(None)));
        assert_ne!(null_value(&ColumnType::Integer), Some(Value::Int(Some(0))));
    }
}
