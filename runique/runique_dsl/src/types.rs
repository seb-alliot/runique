//! What each DSL type decides beyond its SQL column: the form field it gets,
//! the values that field accepts, and the engines that can't store it.
//!
//! The macro, the admin and the forms built from a schema all read this table,
//! so the same field can't be a password in one place and plain text in
//! another, or accept a value its column can't hold.
use crate::ast::{FieldDef, FieldOption, FieldType, FileKind, FormFieldKind};

/// The form field a DSL type is edited with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Widget {
    Text,
    Textarea,
    Richtext,
    Email,
    Url,
    Phone,
    Password,
    /// Whole number, limited to what the Rust field can hold.
    Integer {
        min: i128,
        max: i128,
    },
    Float,
    Decimal,
    Percent,
    Bool,
    Date,
    Time,
    DateTime,
    File(FileKind),
    /// Upload stored as bytes in the column.
    Binary,
    Choice,
    Radio,
    Checkbox,
    Color,
    Slug,
    Uuid,
    Json,
    Ip,
}

/// A database engine, as far as type support goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    Postgres,
    Mysql,
    Sqlite,
}

impl FormFieldKind {
    /// The form field this type is edited with.
    pub fn widget(self) -> Widget {
        use FormFieldKind::*;
        match self {
            Text | Char => Widget::Text,
            Textarea => Widget::Textarea,
            Richtext => Widget::Richtext,
            Email => Widget::Email,
            Url => Widget::Url,
            Phone => Widget::Phone,
            Password => Widget::Password,
            I8 => integer(i8::MIN.into(), i8::MAX.into()),
            I16 => integer(i16::MIN.into(), i16::MAX.into()),
            Int => integer(i32::MIN.into(), i32::MAX.into()),
            Bigint => integer(i64::MIN.into(), i64::MAX.into()),
            U32 => integer(0, u32::MAX.into()),
            U64 => integer(0, u64::MAX.into()),
            Float | F32 => Widget::Float,
            Decimal => Widget::Decimal,
            Percent => Widget::Percent,
            Bool => Widget::Bool,
            Date => Widget::Date,
            Time => Widget::Time,
            Datetime | Timestamp | TimestampTz => Widget::DateTime,
            Image => Widget::File(FileKind::Image),
            Document => Widget::File(FileKind::Document),
            File => Widget::File(FileKind::Any),
            Choice => Widget::Choice,
            Radio => Widget::Radio,
            Checkbox => Widget::Checkbox,
            Color => Widget::Color,
            Slug => Widget::Slug,
            Uuid => Widget::Uuid,
            Json | JsonBinary => Widget::Json,
            Ip => Widget::Ip,
            Binary | VarBinary | Blob => Widget::Binary,
            // No dedicated field yet: edited as text.
            Cidr | MacAddress | Interval => Widget::Text,
        }
    }

    /// Engines that can't read this type back into its Rust field: Postgres
    /// has no unsigned or one-byte integers (sqlx sends an `i8` as `"char"`),
    /// and sqlx only decodes `u64` on MySQL/MariaDB.
    pub fn unsupported_on(self) -> &'static [Engine] {
        use FormFieldKind::*;
        match self {
            I8 | U32 => &[Engine::Postgres],
            U64 => &[Engine::Postgres, Engine::Sqlite],
            _ => &[],
        }
    }
}

impl FormFieldKind {
    /// Largest value a binary column holds, in bytes: its declared length
    /// (`[max_length: n]`), 255 by default like the column itself; `None` for
    /// `blob`, bounded only by the engine.
    pub fn byte_limit(self, max_length: Option<u32>) -> Option<u32> {
        match self {
            FormFieldKind::Binary | FormFieldKind::VarBinary => Some(max_length.unwrap_or(255)),
            _ => None,
        }
    }
}

impl FormFieldKind {
    /// The column this type is stored in. `max_length` sets the length of the
    /// text and binary types that take one; `enum_ref` is the enum a choice
    /// field draws from.
    pub fn column_type(self, max_length: Option<u32>, enum_ref: Option<&syn::Ident>) -> FieldType {
        use FormFieldKind::*;
        match self {
            Text => max_length.map_or(FieldType::String, FieldType::Varchar),
            Email => FieldType::Varchar(254),
            Phone => FieldType::Varchar(max_length.unwrap_or(20)),
            Password | Url | Color | Slug | Image | Document | File => FieldType::String,
            Richtext | Textarea => FieldType::Text,
            Char => FieldType::Char,
            Json => FieldType::Json,
            JsonBinary => FieldType::JsonBinary,
            Int => FieldType::I32,
            Bigint => FieldType::I64,
            I8 => FieldType::I8,
            I16 => FieldType::I16,
            U32 => FieldType::U32,
            U64 => FieldType::U64,
            Float | Percent => FieldType::F64,
            F32 => FieldType::F32,
            Decimal => FieldType::Decimal(None),
            Bool => FieldType::Bool,
            Date => FieldType::Date,
            Time => FieldType::Time,
            Datetime => FieldType::Datetime,
            Timestamp => FieldType::Timestamp,
            TimestampTz => FieldType::TimestampTz,
            Uuid => FieldType::Uuid,
            Ip => FieldType::Inet,
            Cidr => FieldType::Cidr,
            MacAddress => FieldType::MacAddress,
            Interval => FieldType::Interval,
            Choice | Radio | Checkbox => {
                enum_ref.map_or(FieldType::String, |id| FieldType::Enum(id.clone()))
            }
            Binary => FieldType::Binary(max_length),
            VarBinary => FieldType::VarBinary(max_length.unwrap_or(255)),
            Blob => FieldType::Blob,
        }
    }
}

impl FieldDef {
    /// The column this field is stored in — computed from its declared type,
    /// never stored next to it.
    pub fn column_type(&self) -> FieldType {
        let max_length = self.options.iter().find_map(|o| match o {
            FieldOption::MaxLen(n) => Some(*n),
            _ => None,
        });
        self.kind.column_type(max_length, self.enum_ref.as_ref())
    }
}

fn integer(min: i128, max: i128) -> Widget {
    Widget::Integer { min, max }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_bounds_follow_the_rust_type() {
        assert_eq!(FormFieldKind::I8.widget(), integer(-128, 127));
        assert_eq!(FormFieldKind::U64.widget(), integer(0, u64::MAX.into()));
    }

    #[test]
    fn engines_refused_per_type() {
        assert_eq!(FormFieldKind::I8.unsupported_on(), &[Engine::Postgres]);
        assert_eq!(FormFieldKind::U32.unsupported_on(), &[Engine::Postgres]);
        assert_eq!(
            FormFieldKind::U64.unsupported_on(),
            &[Engine::Postgres, Engine::Sqlite]
        );
        assert!(FormFieldKind::Int.unsupported_on().is_empty());
        assert!(FormFieldKind::I16.unsupported_on().is_empty());
    }

    #[test]
    fn byte_limit_is_the_column_length() {
        assert_eq!(FormFieldKind::Binary.byte_limit(None), Some(255));
        assert_eq!(FormFieldKind::VarBinary.byte_limit(Some(8)), Some(8));
        assert_eq!(
            FormFieldKind::Blob.byte_limit(Some(8)),
            None,
            "a blob has no column length"
        );
        assert_eq!(FormFieldKind::Text.byte_limit(Some(8)), None);
    }
}
