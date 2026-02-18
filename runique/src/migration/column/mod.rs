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
        let name_iden: DynIden = sea_query::Alias::new(name.into()).into_iden();
        let variants_iden: Vec<DynIden> = variants
            .into_iter()
            .map(|v| sea_query::Alias::new(v).into_iden())
            .collect();
        self.col_type = ColumnType::Enum {
            name: name_iden,
            variants: variants_iden,
        };
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
}
