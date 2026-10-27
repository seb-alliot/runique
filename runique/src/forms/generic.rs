//! Generic field `GenericField`: unified enum to dispatch to any field type.
use crate::forms::{
    base::{CommonFieldConfig, FormField},
    fields::*,
};
use crate::utils::aliases::ATera;
use crate::{define_enum_field, delegate_to_field};
use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;

define_enum_field!(
    Text => TextField,
    Numeric => NumericField,
    #[serde(skip)]
    File => FileField,
    Binary => BinaryField,
    Boolean => BooleanField,
    Choice => ChoiceField,
    Radio => RadioField,
    Checkbox => CheckboxField,
    Date => DateField,
    Time => TimeField,
    DateTime => DateTimeField,
    Duration => DurationField,
    Color => ColorField,
    Slug => SlugField,
    UUID => UUIDField,
    JSON => JSONField,
    IPAddress => IPAddressField,
    Hidden => HiddenField,
);

/// Type-erased wrapper around any concrete field type (`TextField`,
/// `NumericField`, `FileField`, …), dispatching every [`FormField`] call to
/// the wrapped `field` via `delegate_to_field!`. Lets `Forms` store a single
/// heterogeneous collection of fields.
#[derive(Clone, Serialize, Debug)]
pub struct GenericField {
    pub field: GeneratedField,
}

impl CommonFieldConfig for GenericField {
    fn get_field_config(&self) -> &crate::forms::base::FieldConfig {
        delegate_to_field!(self, get_field_config)
    }

    fn get_field_config_mut(&mut self) -> &mut crate::forms::base::FieldConfig {
        delegate_to_field!(mut self, get_field_config_mut)
    }
}

#[async_trait]
impl FormField for GenericField {
    // --- Getters ---

    fn name(&self) -> &str {
        delegate_to_field!(self, name)
    }

    fn label(&self) -> &str {
        delegate_to_field!(self, label)
    }

    fn value(&self) -> &str {
        delegate_to_field!(self, value)
    }

    fn placeholder(&self) -> &str {
        delegate_to_field!(self, placeholder)
    }

    fn field_type(&self) -> &str {
        delegate_to_field!(self, field_type)
    }

    fn template_name(&self) -> &str {
        delegate_to_field!(self, template_name)
    }

    fn required(&self) -> bool {
        delegate_to_field!(self, required)
    }

    fn error(&self) -> Option<&String> {
        delegate_to_field!(self, error)
    }

    // --- Setters ---

    fn set_name(&mut self, name: &str) {
        delegate_to_field!(mut self, set_name, name)
    }

    fn set_label(&mut self, label: &str) {
        delegate_to_field!(mut self, set_label, label)
    }

    fn set_value(&mut self, value: &str) {
        delegate_to_field!(mut self, set_value, value);
    }

    fn set_submitted_value(&mut self, value: &str) {
        delegate_to_field!(mut self, set_submitted_value, value);
    }

    fn set_type_bounds(&mut self, min: i128, max: i128) -> bool {
        delegate_to_field!(mut self, set_type_bounds, min, max)
    }

    fn bounds(&self) -> crate::forms::base::FieldBounds {
        delegate_to_field!(self, bounds)
    }

    fn cap_max_size(&mut self, bytes: u64) {
        delegate_to_field!(mut self, cap_max_size, bytes);
    }

    fn model_max_size(&self) -> Option<u64> {
        delegate_to_field!(self, model_max_size)
    }

    fn set_max_size_bounded(&mut self, size: crate::forms::fields::FileSize) -> Result<(), String> {
        delegate_to_field!(mut self, set_max_size_bounded, size)
    }

    fn set_placeholder(&mut self, placeholder: &str) {
        delegate_to_field!(mut self, set_placeholder, placeholder)
    }

    fn set_error(&mut self, error: String) {
        delegate_to_field!(mut self, set_error, error)
    }

    fn set_html_attribute(&mut self, key: &str, value: &str) {
        delegate_to_field!(mut self, set_html_attribute, key, value)
    }

    fn set_required(&mut self, required: bool, msg: Option<&str>) {
        delegate_to_field!(mut self, set_required, required, msg)
    }

    fn set_readonly(&mut self, readonly: bool, msg: Option<&str>) {
        delegate_to_field!(mut self, set_readonly, readonly, msg)
    }

    fn set_disabled(&mut self, disabled: bool, msg: Option<&str>) {
        delegate_to_field!(mut self, set_disabled, disabled, msg)
    }

    // --- Business logic ---

    async fn validate(&mut self) -> bool {
        delegate_to_field!(mut self, validate).await
    }

    fn render(&self, tera: &ATera) -> Result<String, String> {
        delegate_to_field!(self, render, tera)
    }

    // --- JSON serialization ---

    fn to_json_value(&self) -> Value {
        delegate_to_field!(self, to_json_value)
    }

    fn to_json_required(&self) -> Value {
        delegate_to_field!(self, to_json_required)
    }

    fn to_json_attributes(&self) -> Value {
        delegate_to_field!(self, to_json_attributes)
    }

    fn to_json_meta(&self) -> Value {
        delegate_to_field!(self, to_json_meta)
    }
    async fn finalize(&mut self) -> Result<(), String> {
        delegate_to_field!(mut self, finalize).await
    }

    fn column_type(&self) -> Option<&sea_orm::sea_query::ColumnType> {
        delegate_to_field!(self, column_type)
    }

    fn accepts_column(&self, column: &sea_orm::sea_query::ColumnType) -> bool {
        accepts(&self.field, column)
    }

    fn apply_column_type(&mut self, column: &sea_orm::sea_query::ColumnType) {
        delegate_to_field!(mut self, apply_column_type, column)
    }

    fn to_value(&self) -> Result<Option<sea_orm::Value>, String> {
        delegate_to_field!(self, to_value)
    }

    fn keeps_when_empty(&self) -> bool {
        delegate_to_field!(self, keeps_when_empty)
    }
}

/// Whether this field can store `column` at all: checked before
/// [`FormField::apply_column_type`]. Every field type is listed, with no
/// catch-all: a new one doesn't compile here until it says which columns it
/// takes.
pub fn accepts(field: &GeneratedField, column: &sea_orm::sea_query::ColumnType) -> bool {
    use crate::forms::base::NumericConfig;
    use sea_orm::sea_query::ColumnType as C;

    let text = matches!(column, C::String(_) | C::Text | C::Char(_));
    let integer = matches!(
        column,
        C::TinyInteger
            | C::SmallInteger
            | C::Integer
            | C::BigInteger
            | C::TinyUnsigned
            | C::SmallUnsigned
            | C::Unsigned
            | C::BigUnsigned
    );
    let float = matches!(column, C::Float | C::Double);
    match field {
        GeneratedField::Text(_)
        | GeneratedField::Color(_)
        | GeneratedField::Slug(_)
        | GeneratedField::IPAddress(_)
        | GeneratedField::Duration(_)
        // A file is stored as its path.
        | GeneratedField::File(_) => text,
        GeneratedField::Numeric(numeric) => match numeric.config {
            NumericConfig::Integer { .. } => integer,
            NumericConfig::Decimal { .. } => matches!(column, C::Decimal(_)),
            NumericConfig::Float { .. }
            | NumericConfig::Percent { .. }
            | NumericConfig::Range { .. } => float,
        },
        GeneratedField::Boolean(_) => matches!(column, C::Boolean),
        GeneratedField::Date(_) => matches!(column, C::Date),
        GeneratedField::Time(_) => matches!(column, C::Time),
        GeneratedField::DateTime(_) => matches!(
            column,
            C::DateTime | C::Timestamp | C::TimestampWithTimeZone
        ),
        GeneratedField::UUID(_) => matches!(column, C::Uuid),
        GeneratedField::JSON(_) => matches!(column, C::Json | C::JsonBinary),
        GeneratedField::Binary(_) => matches!(column, C::Binary(_) | C::VarBinary(_) | C::Blob),
        // An enum's stored value: text, an integer, or a native Postgres enum.
        GeneratedField::Choice(_) | GeneratedField::Radio(_) => {
            text || integer || matches!(column, C::Enum { .. })
        }
        // A list lives in a table of its own: no column holds one.
        GeneratedField::Checkbox(_) => false,
        // Code sets a hidden value, not the user: any column a form can fill.
        GeneratedField::Hidden(_) => {
            text || integer || matches!(column, C::Uuid | C::Boolean | C::Decimal(_))
        }
    }
}
