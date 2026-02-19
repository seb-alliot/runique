use sea_query::ColumnType;
use sea_query::IntoIden;

/// Definition of a simple column
#[derive(Debug, Clone)]
pub struct ColumnDef {
    pub name: String,
    pub col_type: ColumnType,
    pub nullable: bool,
    pub unique: bool,
    pub default: Option<sea_query::Value>,
    pub select_as: Option<String>,
    pub save_as: Option<String>,
    pub ignored: bool,
    pub auto_now: bool,        // created_at: value at creation
    pub auto_now_update: bool, // updated_at: value at each update
    pub enum_variants: Vec<String>,
}

impl ColumnDef {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            col_type: ColumnType::String(sea_query::StringLen::None),
            nullable: false,
            unique: false,
            default: None,
            select_as: None,
            save_as: None,
            ignored: false,
            auto_now: false,
            auto_now_update: false,
            enum_variants: Vec::new(),
        }
    }

    // ── Types ───────────────────────────────────────────────────────────────

    pub fn string(mut self) -> Self {
        self.col_type = ColumnType::String(sea_query::StringLen::None);
        self
    }

    pub fn string_len(mut self, len: u32) -> Self {
        self.col_type = ColumnType::String(sea_query::StringLen::N(len));
        self
    }

    pub fn text(mut self) -> Self {
        self.col_type = ColumnType::Text;
        self
    }

    pub fn integer(mut self) -> Self {
        self.col_type = ColumnType::Integer;
        self
    }

    pub fn big_integer(mut self) -> Self {
        self.col_type = ColumnType::BigInteger;
        self
    }

    pub fn float(mut self) -> Self {
        self.col_type = ColumnType::Float;
        self
    }

    pub fn double(mut self) -> Self {
        self.col_type = ColumnType::Double;
        self
    }

    pub fn boolean(mut self) -> Self {
        self.col_type = ColumnType::Boolean;
        self
    }

    pub fn datetime(mut self) -> Self {
        self.col_type = ColumnType::DateTime;
        self
    }

    pub fn timestamp(mut self) -> Self {
        self.col_type = ColumnType::Timestamp;
        self
    }

    pub fn timestamp_tz(mut self) -> Self {
        self.col_type = ColumnType::TimestampWithTimeZone;
        self
    }

    pub fn date(mut self) -> Self {
        self.col_type = ColumnType::Date;
        self
    }

    pub fn time(mut self) -> Self {
        self.col_type = ColumnType::Time;
        self
    }

    pub fn uuid(mut self) -> Self {
        self.col_type = ColumnType::Uuid;
        self
    }

    pub fn json(mut self) -> Self {
        self.col_type = ColumnType::Json;
        self
    }

    pub fn json_binary(mut self) -> Self {
        self.col_type = ColumnType::JsonBinary;
        self
    }

    pub fn decimal(mut self) -> Self {
        self.col_type = ColumnType::Decimal(None);
        self
    }

    pub fn decimal_len(mut self, precision: u32, scale: u32) -> Self {
        self.col_type = ColumnType::Decimal(Some((precision, scale)));
        self
    }

    pub fn enum_type(mut self, name: impl Into<String>, variants: Vec<String>) -> Self {
        use sea_query::DynIden;
        let name_str = name.into();
        let variants_iden: Vec<DynIden> = variants
            .iter()
            .map(|v| sea_query::Alias::new(v.clone()).into_iden())
            .collect();
        self.col_type = ColumnType::Enum {
            name: sea_query::Alias::new(&name_str).into_iden(),
            variants: variants_iden,
        };
        self.enum_variants = variants; // ← sauvegarde en clair
        self
    }

    // ── Modifiers ───────────────────────────────────────────────────────────

    pub fn required(mut self) -> Self {
        self.nullable = false;
        self
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn default(mut self, value: sea_query::Value) -> Self {
        self.default = Some(value);
        self
    }

    pub fn select_as(mut self, alias: impl Into<String>) -> Self {
        self.select_as = Some(alias.into());
        self
    }

    pub fn save_as(mut self, alias: impl Into<String>) -> Self {
        self.save_as = Some(alias.into());
        self
    }

    pub fn ignore(mut self) -> Self {
        self.ignored = true;
        self
    }

    pub fn auto_now(mut self) -> Self {
        self.col_type = ColumnType::DateTime;
        self.auto_now = true;
        self
    }

    pub fn auto_now_update(mut self) -> Self {
        self.col_type = ColumnType::DateTime;
        self.auto_now_update = true;
        self
    }

    /// Generates the corresponding SeaQuery ColumnDef
    pub fn to_sea_column(&self) -> sea_query::ColumnDef {
        let mut col = sea_query::ColumnDef::new_with_type(
            sea_query::Alias::new(&self.name),
            self.col_type.clone(),
        );

        if self.nullable {
            col.null();
        } else {
            col.not_null();
        }

        if self.unique {
            col.unique_key();
        }

        if let Some(ref val) = self.default {
            col.default(val.clone());
        }

        col
    }
    // ── Form integration ─────────────────────────────────────────────────────────

    /// Convertit la colonne en GenericField.
    /// Retourne `None` si la colonne est auto-exclue.
    pub fn to_form_field(&self) -> Option<crate::forms::generic::GenericField> {
        use crate::forms::base::FormField;
        use crate::forms::fields::{
            boolean::BooleanField,
            choice::ChoiceField,
            datetime::{DateField, DateTimeField, TimeField},
            number::NumericField,
            special::{ColorField, IPAddressField, JSONField, SlugField, UUIDField},
            text::TextField,
        };
        use crate::forms::generic::GenericField;

        if self.auto_now || self.auto_now_update || self.ignored {
            return None;
        }

        let name = self.name.as_str();
        let label = self.format_label();
        let required = !self.nullable;

        let mut field: GenericField = match &self.col_type {
            ColumnType::String(_) => {
                if name == "email" || name.ends_with("_email") {
                    TextField::email(name).into()
                } else if name == "password"
                    || name.ends_with("_password")
                    || name.ends_with("_pwd")
                {
                    TextField::password(name).into()
                } else if name == "url"
                    || name.ends_with("_url")
                    || name.ends_with("_link")
                    || name == "website"
                    || name.ends_with("_website")
                {
                    TextField::url(name).into()
                } else if name == "slug" || name.ends_with("_slug") {
                    SlugField::new(name).into()
                } else if name == "color"
                    || name.ends_with("_color")
                    || name == "colour"
                    || name.ends_with("_colour")
                {
                    ColorField::new(name).into()
                } else if name == "ip" || name.ends_with("_ip") || name.contains("ip_address") {
                    IPAddressField::new(name).into()
                } else {
                    TextField::text(name).into()
                }
            }
            ColumnType::Text => {
                if name.contains("description")
                    || name.contains("bio")
                    || name.contains("content")
                    || name.contains("message")
                    || name.contains("summary")
                    || name.contains("richtext")
                {
                    TextField::richtext(name).into()
                } else {
                    TextField::textarea(name).into()
                }
            }
            ColumnType::Integer
            | ColumnType::BigInteger
            | ColumnType::TinyInteger
            | ColumnType::SmallInteger
            | ColumnType::Unsigned
            | ColumnType::BigUnsigned => NumericField::integer(name).into(),
            ColumnType::Float | ColumnType::Double => NumericField::float(name).into(),
            ColumnType::Decimal(_) => NumericField::decimal(name).into(),
            ColumnType::Boolean => BooleanField::new(name).into(),
            ColumnType::Date => DateField::new(name).into(),
            ColumnType::Time => TimeField::new(name).into(),
            ColumnType::DateTime | ColumnType::Timestamp | ColumnType::TimestampWithTimeZone => {
                DateTimeField::new(name).into()
            }
            ColumnType::Uuid => UUIDField::new(name).into(),
            ColumnType::Enum { .. } => {
                let mut f = ChoiceField::new(name);
                for v in &self.enum_variants {
                    f = f.add_choice(v, v);
                }
                f.into()
            }
            ColumnType::Json | ColumnType::JsonBinary => JSONField::new(name).into(),
            _ => TextField::text(name).into(),
        };

        field.set_label(&label);
        if required {
            field.set_required(true, None);
        }

        Some(field)
    }

    fn format_label(&self) -> String {
        self.name
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
