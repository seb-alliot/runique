// model_macro/src/model/ast.rs — structures representing the parsed DSL

pub enum EnumBackingType {
    /// Detected from `.env`: native Postgres (`CREATE TYPE … AS ENUM`) if engine = Postgres,
    /// otherwise VARCHAR. This is the default behavior when no type is specified.
    Auto,
    I32,
    I64,
}

pub struct EnumVariant {
    pub name: syn::Ident,
    pub value: Option<syn::Lit>,
    pub label: Option<syn::Lit>,
}

impl EnumVariant {
    /// Value stored in database (String): explicit value or variant name.
    pub fn db_str(&self) -> String {
        match &self.value {
            Some(syn::Lit::Str(s)) => s.value(),
            Some(_) | None => self.name.to_string(),
        }
    }

    /// Displayed label (admin form): explicit label, otherwise db_str.
    pub fn display_str(&self) -> String {
        match &self.label {
            Some(syn::Lit::Str(s)) => s.value(),
            _ => self.db_str(),
        }
    }
}

pub struct EnumDef {
    pub name: syn::Ident,
    pub backing_type: EnumBackingType,
    pub variants: Vec<EnumVariant>,
}

pub struct ModelInput {
    pub name: syn::Ident,
    pub table: String,
    pub pk: PkDef,
    pub enums: Vec<EnumDef>,
    pub fields: Vec<FieldDef>,
    pub relations: Vec<RelationDef>,
    pub meta: Option<MetaDef>,
    pub form_fields: Vec<FormFieldDecl>,
}

// ── form_fields: block — semantic types ──────────────────────

pub enum FormFieldKind {
    Text,
    Email,
    Password,
    Richtext,
    Textarea,
    Url,
    Int,
    Float,
    Decimal,
    Percent,
    Bool,
    Date,
    Time,
    Datetime,
    Image,
    Document,
    File,
    Color,
    Slug,
    Uuid,
    Json,
    Ip,
    Choice,
    Radio,
    Bigint,
}

pub enum FormFieldAttr {
    Required,
    Nullable,
    NoHash,
    MaxLength(u32),
    MinLength(u32),
    Min(i64),
    Max(i64),
    MinF(f64),
    MaxF(f64),
    Default(syn::Lit),
    UploadTo(String),
    MaxSize(u64),
    Rows(u32),
    Step(f64),
    /// Reference to an enum declared in `enums:` — used with `choice` and `radio`.
    EnumRef(syn::Ident),
    /// Automatically filled on creation — excludes the field from the form.
    AutoNow,
    /// Automatically filled on every update — excludes the field from the form.
    AutoNowUpdate,
    /// UNIQUE constraint on the SQL column.
    Unique,
}

pub struct FormFieldDecl {
    pub name: syn::Ident,
    pub kind: FormFieldKind,
    pub attrs: Vec<FormFieldAttr>,
}

pub struct PkDef {
    pub name: syn::Ident,
    pub ty: PkType,
}

pub enum PkType {
    I32,
    I64,
    Uuid,
}

pub struct FieldDef {
    pub name: syn::Ident,
    pub ty: FieldType,
    pub options: Vec<FieldOption>,
}

pub enum FieldType {
    String,
    Text,
    Char,
    Varchar(u32),
    I8,
    I16,
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Decimal(Option<(u32, u32)>),
    Bool,
    Date,
    Time,
    Datetime,
    Timestamp,
    TimestampTz,
    Uuid,
    Json,
    JsonBinary,
    Binary(Option<u32>),
    VarBinary(u32),
    Blob,
    Enum(syn::Ident),
    Inet,
    Cidr,
    MacAddress,
    Interval,
}

pub enum FieldOption {
    Required,
    Nullable,
    Unique,
    Index,
    Default(syn::Lit),
    MaxLen(u32),
    MinLen(u32),
    Max(i64),
    Min(i64),
    MaxF(f64),
    MinF(f64),
    AutoNow,
    AutoNowUpdate,
    Readonly,
    SelectAs(String),
    Label(String),
    #[allow(dead_code)]
    Help(String),
    Fk(FkDef),
    File {
        kind: FileKind,
        upload_to: Option<String>,
    },
    MaxSize(u64),
}

pub enum FileKind {
    Image,
    Document,
    Any,
}

pub struct FkDef {
    pub table: syn::Ident,
    pub column: syn::Ident,
    pub action: FkAction,
}

pub enum FkAction {
    Cascade,
    SetNull,
    Restrict,
    SetDefault,
}

pub enum RelationDef {
    BelongsTo {
        model: syn::Ident,
        via: syn::Ident,
    },
    HasMany {
        model: syn::Ident,
        as_name: Option<syn::Ident>,
    },
    HasOne {
        model: syn::Ident,
        as_name: Option<syn::Ident>,
    },
    ManyToMany {
        model: syn::Ident,
        through: syn::Ident,
        via_self: syn::Ident, // ← new macro model
    },
}

pub struct MetaDef {
    pub ordering: Vec<(bool, syn::Ident)>, // (desc, field)
    pub unique_together: Vec<Vec<syn::Ident>>,
    pub verbose_name: Option<String>,
    pub verbose_name_plural: Option<String>,
    #[allow(dead_code)]
    pub abstract_model: bool,
    pub indexes: Vec<Vec<syn::Ident>>,
}
