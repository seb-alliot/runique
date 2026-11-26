//! The type of a column in the database: the closed list of what Runique
//! writes in a migration and reads back from one. A type that is not in
//! this list has no name to be given under.
//!
//! These are columns, not fields, and their names are sea-query's, so that
//! what a field of a model is stored in is what SeaORM reads and writes. A
//! field and its column don't always share a name, and several fields share
//! a column:
//!
//! | field of a model                              | column     |
//! |-----------------------------------------------|------------|
//! | `text`, `email`, `url`, `slug`, `password`... | `String`   |
//! | `textarea`, `richtext`                        | `Text`     |
//! | `float`, `percent`                            | `Double`   |
//! | `f32`                                         | `Float`    |
//! | `timestamp`, `datetime`                       | `DateTime` |
//!
//! The list of fields is [`FieldType`]; [`ColumnType::of_field`] goes from
//! one to the other.
use crate::ast::{EnumBackingType, FieldType, PkType};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ColumnType {
    #[default]
    String,
    Text,
    Char,
    TinyInteger,
    SmallInteger,
    Integer,
    BigInteger,
    Unsigned,
    BigUnsigned,
    Float,
    Double,
    Decimal,
    Boolean,
    DateTime,
    Timestamp,
    TimestampWithTimeZone,
    Date,
    Time,
    Uuid,
    Json,
    JsonBinary,
    Binary,
    VarBinary,
    Blob,
    Inet,
    Cidr,
    MacAddr,
    Interval,
    Enum,
}

impl ColumnType {
    /// Every type, once.
    pub const ALL: &[ColumnType] = &[
        ColumnType::String,
        ColumnType::Text,
        ColumnType::Char,
        ColumnType::TinyInteger,
        ColumnType::SmallInteger,
        ColumnType::Integer,
        ColumnType::BigInteger,
        ColumnType::Unsigned,
        ColumnType::BigUnsigned,
        ColumnType::Float,
        ColumnType::Double,
        ColumnType::Decimal,
        ColumnType::Boolean,
        ColumnType::DateTime,
        ColumnType::Timestamp,
        ColumnType::TimestampWithTimeZone,
        ColumnType::Date,
        ColumnType::Time,
        ColumnType::Uuid,
        ColumnType::Json,
        ColumnType::JsonBinary,
        ColumnType::Binary,
        ColumnType::VarBinary,
        ColumnType::Blob,
        ColumnType::Inet,
        ColumnType::Cidr,
        ColumnType::MacAddr,
        ColumnType::Interval,
        ColumnType::Enum,
    ];

    /// The name of the type, as it is shown to the developer.
    pub fn as_str(self) -> &'static str {
        match self {
            ColumnType::String => "String",
            ColumnType::Text => "Text",
            ColumnType::Char => "Char",
            ColumnType::TinyInteger => "TinyInteger",
            ColumnType::SmallInteger => "SmallInteger",
            ColumnType::Integer => "Integer",
            ColumnType::BigInteger => "BigInteger",
            ColumnType::Unsigned => "Unsigned",
            ColumnType::BigUnsigned => "BigUnsigned",
            ColumnType::Float => "Float",
            ColumnType::Double => "Double",
            ColumnType::Decimal => "Decimal",
            ColumnType::Boolean => "Boolean",
            ColumnType::DateTime => "DateTime",
            ColumnType::Timestamp => "Timestamp",
            ColumnType::TimestampWithTimeZone => "TimestampWithTimeZone",
            ColumnType::Date => "Date",
            ColumnType::Time => "Time",
            ColumnType::Uuid => "Uuid",
            ColumnType::Json => "Json",
            ColumnType::JsonBinary => "JsonBinary",
            ColumnType::Binary => "Binary",
            ColumnType::VarBinary => "VarBinary",
            ColumnType::Blob => "Blob",
            ColumnType::Inet => "Inet",
            ColumnType::Cidr => "Cidr",
            ColumnType::MacAddr => "MacAddr",
            ColumnType::Interval => "Interval",
            ColumnType::Enum => "Enum",
        }
    }

    /// The type of that name; `None` for a name that is not one of them.
    pub fn from_name(name: &str) -> Option<ColumnType> {
        ColumnType::ALL
            .iter()
            .copied()
            .find(|column_type| column_type.as_str() == name)
    }

    /// The `ColumnDef` call a migration writes for this type, as sea-query
    /// takes it, without a length or a precision of the model's:
    /// `integer()`, `date_time()`.
    ///
    /// Three types have no call without an argument in sea-query:
    /// `var_binary` takes a length (255, the default of the column),
    /// `interval` takes its fields and its precision (none), and an enum
    /// column is never written by a method: its type is chosen when the
    /// migration runs, the Postgres enum type there, a string elsewhere.
    /// Asked here, it is the string it is stored as.
    pub fn method(self) -> &'static str {
        match self {
            ColumnType::String => "string()",
            ColumnType::Text => "text()",
            ColumnType::Char => "char()",
            ColumnType::TinyInteger => "tiny_integer()",
            ColumnType::SmallInteger => "small_integer()",
            ColumnType::Integer => "integer()",
            ColumnType::BigInteger => "big_integer()",
            ColumnType::Unsigned => "unsigned()",
            ColumnType::BigUnsigned => "big_unsigned()",
            ColumnType::Float => "float()",
            ColumnType::Double => "double()",
            ColumnType::Decimal => "decimal()",
            ColumnType::Boolean => "boolean()",
            ColumnType::DateTime => "date_time()",
            ColumnType::Timestamp => "timestamp()",
            ColumnType::TimestampWithTimeZone => "timestamp_with_time_zone()",
            ColumnType::Date => "date()",
            ColumnType::Time => "time()",
            ColumnType::Uuid => "uuid()",
            ColumnType::Json => "json()",
            ColumnType::JsonBinary => "json_binary()",
            ColumnType::Binary => "binary()",
            ColumnType::VarBinary => "var_binary(255)",
            ColumnType::Blob => "blob()",
            ColumnType::Inet => "inet()",
            ColumnType::Cidr => "cidr()",
            ColumnType::MacAddr => "mac_address()",
            ColumnType::Interval => "interval(None, None)",
            ColumnType::Enum => "string()",
        }
    }

    /// The type a column definition of a migration file gives, from the names
    /// of the methods it calls: the reverse of [`ColumnType::method`], with
    /// the spellings that carry a length or a precision (`string_len`,
    /// `decimal_len`).
    ///
    /// A definition that calls no type method is a `String`: an enum column
    /// has none, its type being chosen when the migration runs. A method that
    /// is not in the list is read the same way.
    ///
    /// Only the type is read here. What a field of a model bounds its values
    /// with (`min`, `max`, `min_length`) never reaches the column: of these,
    /// the column holds a length (`max_length`) and a decimal's precision.
    pub fn from_methods(methods: &[String]) -> ColumnType {
        let has = |name: &str| methods.iter().any(|method| method == name);
        // Binaries
        if has("blob") {
            ColumnType::Blob
        } else if has("binary") || has("binary_len") {
            ColumnType::Binary
        } else if has("var_binary") {
            ColumnType::VarBinary
        }
        // Texts
        else if has("text") {
            ColumnType::Text
        } else if has("char") || has("char_len") {
            ColumnType::Char
        }
        // Integers (order: most specific to most general)
        else if has("tiny_integer") {
            ColumnType::TinyInteger
        } else if has("small_integer") {
            ColumnType::SmallInteger
        } else if has("big_unsigned") {
            ColumnType::BigUnsigned
        } else if has("unsigned") {
            ColumnType::Unsigned
        } else if has("big_integer") {
            ColumnType::BigInteger
        } else if has("integer") {
            ColumnType::Integer
        }
        // Numerics
        else if has("float") {
            ColumnType::Float
        } else if has("double") {
            ColumnType::Double
        } else if has("decimal") || has("decimal_len") {
            ColumnType::Decimal
        }
        // Boolean
        else if has("boolean") {
            ColumnType::Boolean
        }
        // Date/Time
        else if has("timestamp_tz") || has("timestamp_with_time_zone") {
            ColumnType::TimestampWithTimeZone
        } else if has("timestamp") {
            ColumnType::Timestamp
        } else if has("date_time") || has("auto_now") || has("auto_now_update") {
            ColumnType::DateTime
        } else if has("date") {
            ColumnType::Date
        } else if has("time") {
            ColumnType::Time
        }
        // UUID
        else if has("uuid") {
            ColumnType::Uuid
        }
        // JSON
        else if has("json_binary") {
            ColumnType::JsonBinary
        } else if has("json") {
            ColumnType::Json
        }
        // PostgreSQL specific
        else if has("inet") {
            ColumnType::Inet
        } else if has("cidr") {
            ColumnType::Cidr
        } else if has("mac_address") {
            ColumnType::MacAddr
        } else if has("interval") {
            ColumnType::Interval
        }
        // Enum: `enumeration` is sea-query's; `enum_type` is what older
        // snapshots hold, which are read and never compiled.
        else if has("enum_type") || has("enumeration") {
            ColumnType::Enum
        }
        // What is left is a string: by its own methods (`string`,
        // `string_len`), or with no type method at all, as an enum column,
        // which is a string off Postgres.
        else {
            ColumnType::String
        }
    }

    /// The column a field of a model is stored in. `ip`/`cidr`/`mac_address`/
    /// `interval` are strings: their native types exist only on Postgres. An
    /// enum is a string, or the integer its values are declared with
    /// (`enum_backing`).
    pub fn of_field(field: &FieldType, enum_backing: Option<&EnumBackingType>) -> ColumnType {
        match field {
            FieldType::String | FieldType::Varchar(_) | FieldType::Char => ColumnType::String,
            FieldType::Text => ColumnType::Text,
            FieldType::I8 => ColumnType::TinyInteger,
            FieldType::I16 => ColumnType::SmallInteger,
            FieldType::I32 => ColumnType::Integer,
            FieldType::I64 => ColumnType::BigInteger,
            FieldType::U32 => ColumnType::Unsigned,
            FieldType::U64 => ColumnType::BigUnsigned,
            FieldType::F32 => ColumnType::Float,
            FieldType::F64 => ColumnType::Double,
            FieldType::Decimal(_) => ColumnType::Decimal,
            FieldType::Bool => ColumnType::Boolean,
            FieldType::Date => ColumnType::Date,
            FieldType::Time => ColumnType::Time,
            FieldType::Datetime | FieldType::Timestamp => ColumnType::DateTime,
            FieldType::TimestampTz => ColumnType::TimestampWithTimeZone,
            FieldType::Uuid => ColumnType::Uuid,
            FieldType::Json | FieldType::JsonBinary => ColumnType::Json,
            FieldType::Binary(_) => ColumnType::Binary,
            FieldType::VarBinary(_) => ColumnType::VarBinary,
            FieldType::Blob => ColumnType::Blob,
            FieldType::Inet | FieldType::Cidr | FieldType::MacAddress | FieldType::Interval => {
                ColumnType::String
            }
            FieldType::Enum(_) => match enum_backing {
                Some(EnumBackingType::I8) => ColumnType::TinyInteger,
                Some(EnumBackingType::I16) => ColumnType::SmallInteger,
                Some(EnumBackingType::I32) => ColumnType::Integer,
                Some(EnumBackingType::I64) => ColumnType::BigInteger,
                _ => ColumnType::String,
            },
        }
    }

    /// The column of a primary key.
    pub fn of_pk(pk: PkType) -> ColumnType {
        match pk {
            PkType::I32 => ColumnType::Integer,
            PkType::I64 => ColumnType::BigInteger,
            PkType::Uuid => ColumnType::Uuid,
        }
    }
}

impl fmt::Display for ColumnType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq<str> for ColumnType {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for ColumnType {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
