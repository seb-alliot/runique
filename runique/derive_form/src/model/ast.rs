// model_macro/src/model/ast.rs — structures qui représentent le DSL parsé

pub struct ModelInput {
    pub name: syn::Ident,
    pub table: String,
    pub pk: PkDef,
    pub fields: Vec<FieldDef>,
    pub relations: Vec<RelationDef>,
    pub meta: Option<MetaDef>,
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
    Enum(Vec<syn::Ident>),
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
    #[allow(dead_code)]
    Label(String),
    #[allow(dead_code)]
    Help(String),
    Fk(FkDef),
    File { kind: FileKind, upload_to: Option<String> },
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
