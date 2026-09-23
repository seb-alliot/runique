//! Converts a `FormFieldDecl` (anonymous block v2) into an equivalent SQL
//! `FieldDef`. SQL types are inferred from semantic types.
use crate::model::ast::{FieldDef, FieldOption, FieldType, FormFieldAttr, FormFieldDecl};

pub(crate) fn form_field_to_field_def(ff: &FormFieldDecl) -> FieldDef {
    use crate::model::ast::{FileKind, FormFieldAttr::*, FormFieldKind::*};

    let is_required = ff.attrs.iter().any(|a| matches!(a, Required));
    let is_nullable = ff.attrs.iter().any(|a| matches!(a, Nullable)) || !is_required; // without required -> implicit nullable

    let max_len = ff
        .attrs
        .iter()
        .find_map(|a| if let MaxLength(n) = a { Some(*n) } else { None });
    let default = ff.attrs.iter().find_map(|a| {
        if let Default(lit) = a {
            Some(lit.clone())
        } else {
            None
        }
    });
    let upload_to = ff.attrs.iter().find_map(|a| {
        if let UploadTo(p) = a {
            Some(p.clone())
        } else {
            None
        }
    });
    let enum_ref = ff.attrs.iter().find_map(|a| {
        if let EnumRef(id) = a {
            Some(id.clone())
        } else {
            None
        }
    });

    let ty = match &ff.kind {
        Text => {
            if let Some(n) = max_len {
                FieldType::Varchar(n)
            } else {
                FieldType::String
            }
        }
        Email => FieldType::Varchar(254),
        Password => FieldType::String,
        Richtext | Textarea => FieldType::Text,
        Json => FieldType::Json,
        Url => FieldType::String,
        Int => FieldType::I32,
        Float => FieldType::F64,
        Decimal => FieldType::Decimal(None),
        Percent => FieldType::F64,
        Bool => FieldType::Bool,
        Date => FieldType::Date,
        Time => FieldType::Time,
        Datetime => FieldType::Datetime,
        Uuid => FieldType::Uuid,
        Ip => FieldType::Inet,
        Color | Slug => FieldType::String,
        Image | Document | File => FieldType::String,
        Choice | Radio | Checkbox => {
            if let Some(ident) = enum_ref {
                FieldType::Enum(ident)
            } else {
                FieldType::String
            }
        }
        Bigint => FieldType::I64,
        Phone => {
            if let Some(n) = max_len {
                FieldType::Varchar(n)
            } else {
                FieldType::Varchar(20)
            }
        }
        Char => FieldType::Char,
        I8 => FieldType::I8,
        I16 => FieldType::I16,
        U32 => FieldType::U32,
        U64 => FieldType::U64,
        F32 => FieldType::F32,
        Timestamp => FieldType::Timestamp,
        TimestampTz => FieldType::TimestampTz,
        JsonBinary => FieldType::JsonBinary,
        // Same mechanism as `text` + `max_length` → `Varchar(n)`: the byte length
        // rides on the already-parsed `max_length` attribute, no new syntax needed.
        Binary => FieldType::Binary(max_len),
        VarBinary => FieldType::VarBinary(max_len.unwrap_or(255)),
        Blob => FieldType::Blob,
        Cidr => FieldType::Cidr,
        MacAddress => FieldType::MacAddress,
        Interval => FieldType::Interval,
    };

    let is_auto_now = ff.attrs.iter().any(|a| matches!(a, AutoNow));
    let is_auto_now_update = ff.attrs.iter().any(|a| matches!(a, AutoNowUpdate));

    let mut options: Vec<FieldOption> = Vec::new();
    if is_auto_now {
        options.push(FieldOption::AutoNow);
    } else if is_auto_now_update {
        options.push(FieldOption::AutoNowUpdate);
    } else if is_required && !is_nullable {
        options.push(FieldOption::Required);
    } else if is_nullable && !is_required {
        options.push(FieldOption::Nullable);
    }
    if ff.attrs.iter().any(|a| matches!(a, FormFieldAttr::Unique)) {
        options.push(FieldOption::Unique);
    }
    if let Some(lit) = default {
        options.push(FieldOption::Default(lit));
    }
    if let Some(path) = upload_to {
        let file_kind = match &ff.kind {
            Image => FileKind::Image,
            Document => FileKind::Document,
            _ => FileKind::Any,
        };
        options.push(FieldOption::File {
            kind: file_kind,
            upload_to: Some(path),
        });
    }
    if let Some(FormFieldAttr::MaxLength(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxLength(_)))
    {
        options.push(FieldOption::MaxLen(*n));
    }
    if let Some(FormFieldAttr::MinLength(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MinLength(_)))
    {
        options.push(FieldOption::MinLen(*n));
    }
    if let Some(FormFieldAttr::Min(n)) =
        ff.attrs.iter().find(|a| matches!(a, FormFieldAttr::Min(_)))
    {
        options.push(FieldOption::Min(*n));
    }
    if let Some(FormFieldAttr::Max(n)) =
        ff.attrs.iter().find(|a| matches!(a, FormFieldAttr::Max(_)))
    {
        options.push(FieldOption::Max(*n));
    }
    if let Some(FormFieldAttr::MinF(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MinF(_)))
    {
        options.push(FieldOption::MinF(*n));
    }
    if let Some(FormFieldAttr::MaxF(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxF(_)))
    {
        options.push(FieldOption::MaxF(*n));
    }
    if let Some(FormFieldAttr::MaxSize(n)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::MaxSize(_)))
    {
        options.push(FieldOption::MaxSize(*n));
    }
    if ff
        .attrs
        .iter()
        .any(|a| matches!(a, FormFieldAttr::Readonly))
    {
        options.push(FieldOption::Readonly);
    }
    if let Some(FormFieldAttr::Label(s)) = ff
        .attrs
        .iter()
        .find(|a| matches!(a, FormFieldAttr::Label(_)))
    {
        options.push(FieldOption::Label(s.clone()));
    }
    if let Some(FormFieldAttr::Fk(fk)) = ff.attrs.iter().find(|a| matches!(a, FormFieldAttr::Fk(_)))
    {
        options.push(FieldOption::Fk(fk.clone()));
    }

    FieldDef {
        name: ff.name.clone(),
        ty,
        options,
    }
}
