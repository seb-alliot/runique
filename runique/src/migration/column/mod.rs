//! Table column definition — type, constraints, validation, and SeaQuery generation.
//!
//! [`ColumnDef`] is the entry point. It follows the builder pattern:
//! `ColumnDef::new("slug").varchar(200).unique().nullable()`.
//! The [`ColumnDef::to_sea_column`] method produces the corresponding [`sea_query::ColumnDef`].
//! The [`ColumnDef::to_form_field`] method automatically generates the appropriate form field.
use sea_query::{ColumnType, IntoIden};

/// Upload kind for a file column — drives the form widget and the allowed
/// extension whitelist when the schema rebuilds a `FileField`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileKind {
    Image,
    Document,
    Any,
}

/// Complete table column definition.
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
    pub max_length: Option<u32>,
    pub min_length: Option<u32>,
    pub max_value: Option<i64>,
    pub min_value: Option<i64>,
    pub max_float: Option<f64>,
    pub min_float: Option<f64>,
    /// File upload metadata — pure form concern, ignored by `to_sea_column`.
    pub is_file: bool,
    pub file_kind: Option<FileKind>,
    pub max_size: Option<u64>, // bytes
}

impl ColumnDef {
    /// Creates a column named `name`, defaulting to an unbounded, non-nullable
    /// `VARCHAR` with no constraints.
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
            max_length: None,
            min_length: None,
            max_value: None,
            min_value: None,
            max_float: None,
            min_float: None,
            is_file: false,
            file_kind: None,
            max_size: None,
        }
    }

    // __ size of size
    /// Sets the column type to a tiny (8-bit) integer.
    pub fn tiny_integer(mut self) -> Self {
        self.col_type = ColumnType::TinyInteger;
        self
    }

    /// Sets the column type to a small (16-bit) integer.
    pub fn small_integer(mut self) -> Self {
        self.col_type = ColumnType::SmallInteger;
        self
    }

    /// Sets the column type to an unsigned 32-bit integer.
    pub fn unsigned(mut self) -> Self {
        self.col_type = ColumnType::Unsigned;
        self
    }

    /// Sets the column type to an unsigned 64-bit integer.
    pub fn big_unsigned(mut self) -> Self {
        self.col_type = ColumnType::BigUnsigned;
        self
    }

    // ── Types ───────────────────────────────────────────────────────────────
    /// Binary field with default length of 255 bytes.
    #[doc = include_str!("../../../doc-tests/migration/column_binary.md")]
    pub fn binary(mut self) -> Self {
        self.col_type = ColumnType::Binary(255);
        self
    }

    /// Binary field with custom length.
    ///
    /// # Examples
    ///
    /// ```rust, ignore
    /// use crate::runique::migration::column::ColumnDef;
    /// ColumnDef::new("short_id").binary_len(16);
    /// ColumnDef::new("sha256_hash").binary_len(32);
    /// ColumnDef::new("sha512_hash").binary_len(64);
    /// ```
    pub fn binary_len(mut self, len: u32) -> Self {
        self.col_type = ColumnType::Binary(len);
        self
    }

    /// Sets the column type to a variable-length binary column of `len` bytes.
    pub fn var_binary(mut self, len: u32) -> Self {
        self.col_type = ColumnType::VarBinary(sea_query::StringLen::N(len));
        self
    }

    /// Sets the column type to an unbounded binary blob.
    pub fn blob(mut self) -> Self {
        self.col_type = ColumnType::Blob;
        self
    }

    /// Sets the column type to a fixed-length `CHAR` with no explicit length.
    pub fn char(mut self) -> Self {
        self.col_type = ColumnType::Char(None);
        self
    }

    /// Sets the column type to a fixed-length `CHAR(len)`.
    pub fn char_len(mut self, len: u32) -> Self {
        self.col_type = ColumnType::Char(Some(len));
        self
    }

    /// Sets the column type to an unbounded `VARCHAR` (the default).
    pub fn string(mut self) -> Self {
        self.col_type = ColumnType::String(sea_query::StringLen::None);
        self
    }

    /// Sets the column type to a `VARCHAR(len)`.
    pub fn varchar(mut self, len: u32) -> Self {
        self.col_type = ColumnType::String(sea_query::StringLen::N(len));
        self
    }

    /// Sets the column type to an unbounded `TEXT`.
    pub fn text(mut self) -> Self {
        self.col_type = ColumnType::Text;
        self
    }

    /// Sets the column type to a 32-bit integer.
    pub fn integer(mut self) -> Self {
        self.col_type = ColumnType::Integer;
        self
    }

    /// Sets the column type to a 64-bit integer.
    pub fn big_integer(mut self) -> Self {
        self.col_type = ColumnType::BigInteger;
        self
    }

    /// Sets the column type to a single-precision float.
    pub fn float(mut self) -> Self {
        self.col_type = ColumnType::Float;
        self
    }

    /// Sets the column type to a double-precision float.
    pub fn double(mut self) -> Self {
        self.col_type = ColumnType::Double;
        self
    }

    /// Sets the column type to a boolean.
    pub fn boolean(mut self) -> Self {
        self.col_type = ColumnType::Boolean;
        self
    }

    /// Sets the column type to a timezone-less date-and-time value.
    pub fn datetime(mut self) -> Self {
        self.col_type = ColumnType::DateTime;
        self
    }

    /// Sets the column type to a SQL `TIMESTAMP`.
    pub fn timestamp(mut self) -> Self {
        self.col_type = ColumnType::Timestamp;
        self
    }

    /// Sets the column type to a `TIMESTAMP WITH TIME ZONE`.
    pub fn timestamp_tz(mut self) -> Self {
        self.col_type = ColumnType::TimestampWithTimeZone;
        self
    }

    /// Sets the column type to a date with no time component.
    pub fn date(mut self) -> Self {
        self.col_type = ColumnType::Date;
        self
    }

    /// Sets the column type to a time with no date component.
    pub fn time(mut self) -> Self {
        self.col_type = ColumnType::Time;
        self
    }

    /// Sets the column type to a UUID.
    pub fn uuid(mut self) -> Self {
        self.col_type = ColumnType::Uuid;
        self
    }

    /// Sets the column type to JSON stored as text.
    pub fn json(mut self) -> Self {
        self.col_type = ColumnType::Json;
        self
    }

    /// Sets the column type to JSON stored in a binary format (e.g. Postgres `JSONB`).
    pub fn json_binary(mut self) -> Self {
        self.col_type = ColumnType::JsonBinary;
        self
    }

    /// Sets the column type to a decimal with no explicit precision/scale.
    pub fn decimal(mut self) -> Self {
        self.col_type = ColumnType::Decimal(None);
        self
    }

    /// Sets the column type to a decimal with explicit `precision` (total digits)
    /// and `scale` (digits after the decimal point).
    pub fn decimal_len(mut self, precision: u32, scale: u32) -> Self {
        self.col_type = ColumnType::Decimal(Some((precision, scale)));
        self
    }

    /// Sets the column type to a native SQL enum named `name` with the given
    /// `variants`, and records the variant list on `enum_variants` for
    /// downstream form-field generation.
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
        self.enum_variants = variants; // ← store as plain string
        self
    }

    // ── Modifiers ───────────────────────────────────────────────────────────
    /// Sets the maximum length validation used when rendering the corresponding form field.
    pub fn max_len(mut self, len: u32) -> Self {
        self.max_length = Some(len);
        self
    }

    /// Sets the minimum length validation used when rendering the corresponding form field.
    pub fn min_len(mut self, len: u32) -> Self {
        self.min_length = Some(len);
        self
    }

    /// Sets the maximum integer value validation used when rendering the corresponding form field.
    pub fn max_i64(mut self, val: i64) -> Self {
        self.max_value = Some(val);
        self
    }

    /// Sets the minimum integer value validation used when rendering the corresponding form field.
    pub fn min_i64(mut self, val: i64) -> Self {
        self.min_value = Some(val);
        self
    }

    /// Sets the maximum float value validation used when rendering the corresponding form field.
    pub fn max_f64(mut self, val: f64) -> Self {
        self.max_float = Some(val);
        self
    }

    /// Sets the minimum float value validation used when rendering the corresponding form field.
    pub fn min_f64(mut self, val: f64) -> Self {
        self.min_float = Some(val);
        self
    }

    /// Marks the column `NOT NULL` (the default).
    pub fn required(mut self) -> Self {
        self.nullable = false;
        self
    }

    /// Marks the column nullable.
    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    /// Adds a unique constraint on the column.
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    /// Sets a literal `DEFAULT` value for the column.
    pub fn default(mut self, value: sea_query::Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets an alternate column name to read from when selecting (e.g. a
    /// legacy column name kept in the DB but exposed under a new field name).
    pub fn select_as(mut self, alias: impl Into<String>) -> Self {
        self.select_as = Some(alias.into());
        self
    }

    /// Sets an alternate column name to write to when saving.
    pub fn save_as(mut self, alias: impl Into<String>) -> Self {
        self.save_as = Some(alias.into());
        self
    }

    /// Excludes the column from generated SQL (`to_sea_column`) and from
    /// generated form fields — it exists on the model but not in the schema.
    pub fn ignore(mut self) -> Self {
        self.ignored = true;
        self
    }

    /// Marks the column as a file upload of the given kind. Drives the
    /// `FileField` widget and allowed extensions when rebuilt from the schema.
    pub fn file(mut self, kind: FileKind) -> Self {
        self.is_file = true;
        self.file_kind = Some(kind);
        self
    }

    /// Model-defined upload ceiling in bytes. Bounds any form-level override.
    pub fn max_size_bytes(mut self, bytes: u64) -> Self {
        self.max_size = Some(bytes);
        self
    }

    /// Marks the column as a `created_at`-style timestamp: set once, at insertion.
    pub fn auto_now(mut self) -> Self {
        self.col_type = ColumnType::DateTime;
        self.auto_now = true;
        self
    }

    /// Marks the column as an `updated_at`-style timestamp: refreshed on every update.
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
        } else if self.auto_now {
            // created_at: default value on insertion
            col.extra("DEFAULT CURRENT_TIMESTAMP".to_string());
        } else if self.auto_now_update {
            // updated_at: ON UPDATE for MySQL; trigger handled separately for Postgres
            col.extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string());
        }

        col
    }

    //__ variant of postgres
    /// Sets the column type to a PostgreSQL `INET` address.
    pub fn inet(mut self) -> Self {
        self.col_type = ColumnType::Inet;
        self
    }

    /// Sets the column type to a PostgreSQL `CIDR` network.
    pub fn cidr(mut self) -> Self {
        self.col_type = ColumnType::Cidr;
        self
    }

    /// Sets the column type to a PostgreSQL `MACADDR`.
    pub fn mac_address(mut self) -> Self {
        self.col_type = ColumnType::MacAddr;
        self
    }

    /// Sets the column type to a PostgreSQL `INTERVAL`.
    pub fn interval(mut self) -> Self {
        self.col_type = ColumnType::Interval(None, None);
        self
    }
    // ── Form integration ─────────────────────────────────────────────────────────

    /// Converts the column to a GenericField.
    /// Returns `None` if the column is auto-excluded.
    pub fn to_form_field(&self) -> Option<crate::forms::generic::GenericField> {
        use crate::forms::base::FormField;
        use crate::forms::fields::{
            FileSize,
            boolean::BooleanField,
            choice::ChoiceField,
            datetime::{DateField, DateTimeField, TimeField},
            file::FileField,
            number::NumericField,
            special::{ColorField, IPAddressField, JSONField, SlugField, UUIDField},
            text::TextField,
        };
        use crate::forms::generic::GenericField;

        if self.ignored {
            return None;
        }

        let name = self.name.as_str();
        let label = self.format_label();
        let required = !self.nullable;

        // A file column is stored as String at the SQL level, so the file
        // marker must be checked before the col_type match would route it to a
        // TextField. `max_size` sets the model ceiling, which later bounds any
        // form-level override via `set_max_size_bounded`.
        let mut field: GenericField = if self.is_file {
            let mut ff = match self.file_kind {
                Some(FileKind::Image) => FileField::image(name),
                Some(FileKind::Document) => FileField::document(name),
                Some(FileKind::Any) | None => FileField::any(name),
            };
            if let Some(bytes) = self.max_size {
                ff = ff.max_size(FileSize::bytes(bytes));
            }
            ff.into()
        } else {
            match &self.col_type {
                ColumnType::String(_) => {
                    if name == "email" || name.ends_with("_email") {
                        let mut tf = TextField::email(name);
                        if let Some(max_model) = self.max_length {
                            let current = tf.config.max_length.as_ref().map(|c| c.value);
                            let effective = match current {
                                Some(f) => max_model.min(f),
                                None => max_model,
                            };
                            tf = tf.max_length(effective, "Too long");
                        }
                        tf.into()
                    } else if name == "password"
                        || name.ends_with("_password")
                        || name.ends_with("_pwd")
                    {
                        let mut tf = TextField::password(name);
                        if let Some(max_model) = self.max_length {
                            let current = tf.config.max_length.as_ref().map(|c| c.value);
                            let effective = match current {
                                Some(f) => max_model.min(f),
                                None => max_model,
                            };
                            tf = tf.max_length(effective, "Too long");
                        }
                        tf.into()
                    } else if name == "url"
                        || name.ends_with("_url")
                        || name == "website"
                        || name.ends_with("_website")
                        || name.contains("http")
                    {
                        let mut tf = TextField::url(name);
                        if let Some(max_model) = self.max_length {
                            let current = tf.config.max_length.as_ref().map(|c| c.value);
                            let effective = match current {
                                Some(f) => max_model.min(f),
                                None => max_model,
                            };
                            tf = tf.max_length(effective, "Too long");
                        }
                        tf.into()
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
                        let mut tf = TextField::text(name);
                        if let Some(max_model) = self.max_length {
                            let current = tf.config.max_length.as_ref().map(|c| c.value);
                            let effective = match current {
                                Some(f) => max_model.min(f),
                                None => max_model,
                            };
                            tf = tf.max_length(effective, "Too long");
                        }
                        tf.into()
                    }
                }
                ColumnType::Text => {
                    let mut tf = if name.contains("description")
                        || name.contains("bio")
                        || name.contains("content")
                        || name.contains("message")
                        || name.contains("summary")
                        || name.contains("richtext")
                    {
                        TextField::richtext(name)
                    } else {
                        TextField::textarea(name)
                    };
                    if let Some(max_model) = self.max_length {
                        let current = tf.config.max_length.as_ref().map(|c| c.value);
                        let effective = match current {
                            Some(f) => max_model.min(f),
                            None => max_model,
                        };
                        tf = tf.max_length(effective, "Too long");
                    }
                    tf.into()
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
                ColumnType::DateTime
                | ColumnType::Timestamp
                | ColumnType::TimestampWithTimeZone => DateTimeField::new(name).into(),
                ColumnType::Uuid => UUIDField::new(name).into(),
                ColumnType::Enum { .. } => {
                    let mut f = ChoiceField::new(name);
                    for v in &self.enum_variants {
                        f = f.add_choice(v, v);
                    }
                    f.into()
                }
                ColumnType::Json | ColumnType::JsonBinary => JSONField::new(name).into(),
                ColumnType::Char(_) => TextField::text(name).into(),
                // Type non géré (binary/blob/inet/cidr/interval…) : dégradé en champ texte.
                // Loggé pour rester visible (dégradation silencieuse sinon), pas une erreur fatale.
                other => {
                    tracing::debug!(
                        field = %name,
                        col_type = ?other,
                        "to_form_field: type de colonne non géré → TextField par défaut"
                    );
                    TextField::text(name).into()
                }
            }
        };

        field.set_label(&label);
        if required && !self.auto_now && !self.auto_now_update {
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
