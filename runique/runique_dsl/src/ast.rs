//! Structures representing the parsed `model!{}` / `extend!{}` DSL.

pub enum EnumBackingType {
    /// Detected from `.env`: native Postgres (`CREATE TYPE … AS ENUM`) if engine = Postgres,
    /// otherwise VARCHAR. This is the default behavior when no type is specified.
    Auto,
    I8,
    I16,
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

    /// Stored value of a variant of an `i32`/`i64` enum. The parser refuses
    /// such an enum unless every variant has an integer value that fits.
    pub fn int_value(&self) -> Option<i64> {
        match &self.value {
            Some(syn::Lit::Int(n)) => n.base10_parse().ok(),
            _ => None,
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
    /// `multichoice` / `checkbox` fields: a list of enum values, stored in
    /// their own table, never a column of this one — kept out of `fields`
    /// and `form_fields`.
    pub lists: Vec<FormFieldDecl>,
}

// ── form_fields: block — semantic types ──────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
    /// A list of enum values, edited as checkboxes.
    Checkbox,
    /// A list of enum values, edited as a multiple select.
    Multichoice,
    Bigint,
    Phone,
    Char,
    I8,
    I16,
    U32,
    U64,
    F32,
    Timestamp,
    TimestampTz,
    JsonBinary,
    Binary,
    VarBinary,
    Blob,
    Cidr,
    MacAddress,
    Interval,
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
    /// Excludes the column from generated migrations (Rust field exists, `derive_form`
    /// doesn't manage it) — distinct from `#[form]`'s `field_readonly()`, which only
    /// affects HTML rendering of a specific form instance.
    Readonly,
    /// Overrides the default auto-generated form label for this field.
    Label(String),
    /// Field present in SQL schema but excluded from generated forms.
    Skip,
    /// `renamed_from: "old"` — migration-only: the column used to be called
    /// `old`, so `makemigrations` renames it instead of dropping and re-adding it.
    RenamedFrom(String),
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
    /// The DSL type the field was declared with — see [`crate::types`].
    pub kind: FormFieldKind,
    /// The enum a `choice` / `radio` / `checkbox` field draws from (`[enum(X)]`).
    pub enum_ref: Option<syn::Ident>,
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
    Label(String),
    File {
        kind: FileKind,
        upload_to: Option<String>,
    },
    MaxSize(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileKind {
    Image,
    Document,
    Any,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FkAction {
    NoAction,
    Cascade,
    SetNull,
    Restrict,
    SetDefault,
}

pub enum RelationDef {
    /// `belongs_to: Model via column [on_delete, on_update]` — both actions
    /// default to `NoAction`.
    BelongsTo {
        model: syn::Ident,
        via: syn::Ident,
        on_delete: FkAction,
        on_update: FkAction,
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
    pub indexes: Vec<Vec<syn::Ident>>,
}

#[cfg(test)]
mod tests {
    use super::EnumDef;

    fn variants(src: &str) -> Vec<(String, String)> {
        syn::parse_str::<EnumDef>(src)
            .expect("parses")
            .variants
            .iter()
            .map(|v| (v.db_str(), v.display_str()))
            .collect()
    }

    #[test]
    fn stored_value_and_label_of_a_variant() {
        assert_eq!(
            variants(r#"Status: [Draft, Active: "Actif", Gone=("gone", "Parti")]"#),
            [
                ("Draft".to_string(), "Draft".to_string()),
                ("Active".to_string(), "Actif".to_string()),
                ("gone".to_string(), "Parti".to_string()),
            ]
        );
    }
}
