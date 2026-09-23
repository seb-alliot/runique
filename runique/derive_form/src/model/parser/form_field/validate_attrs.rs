//! Cross-checks a `FormFieldDecl`'s parsed attributes against its kind
//! (e.g. `max_length` only makes sense on textual types).
use crate::model::ast::{FormFieldAttr, FormFieldKind};
use syn::{Ident, Result};

use super::attr_name::attr_name_str;

pub(super) fn validate_form_field_attrs(
    name: &Ident,
    kind_ident: &Ident,
    kind: &FormFieldKind,
    attrs: &[FormFieldAttr],
) -> Result<()> {
    use FormFieldAttr::*;
    use FormFieldKind::*;

    let kind_name = kind_ident.to_string();

    for attr in attrs {
        let valid = match (attr, kind) {
            // required / nullable / readonly / label — universal
            (Required, _) => true,
            (Nullable, _) => true,
            (Readonly, _) => true,
            (Label(_), _) => true,
            (
                EnumRef(_),
                FormFieldKind::Choice | FormFieldKind::Radio | FormFieldKind::Checkbox,
            ) => true,
            (EnumRef(_), _) => false,

            // no_hash — password only
            (NoHash, Password) => true,
            (NoHash, _) => false,

            // max_length / min_length — textual types (also byte length for binary/var_binary)
            (
                MaxLength(_),
                Text | Email | Password | Richtext | Textarea | Url | Phone | Char | Binary
                | VarBinary,
            ) => true,
            (MaxLength(_), _) => false,
            (
                MinLength(_),
                Text | Email | Password | Richtext | Textarea | Url | Phone | Char | Binary
                | VarBinary,
            ) => true,
            (MinLength(_), _) => false,

            // min / max integer — integer types only
            (Min(_), Int | Bigint | I8 | I16 | U32 | U64) => true,
            (Min(_), _) => false,
            (Max(_), Int | Bigint | I8 | I16 | U32 | U64) => true,
            (Max(_), _) => false,

            // min_f / max_f float — float, decimal
            (MinF(_), Float | Decimal | F32) => true,
            (MinF(_), _) => false,
            (MaxF(_), Float | Decimal | F32) => true,
            (MaxF(_), _) => false,

            // default — all except files
            (Default(_), Image | Document | File) => false,
            (Default(_), _) => true,

            // upload_to / max_size - files only
            (UploadTo(_), Image | Document | File) => true,
            (UploadTo(_), _) => false,
            (MaxSize(_), Image | Document | File) => true,
            (MaxSize(_), _) => false,

            // rows — multiline types
            (Rows(_), Richtext | Textarea | Json) => true,
            (Rows(_), _) => false,

            // step — float, decimal, percent
            (Step(_), Float | Decimal | Percent) => true,
            (Step(_), _) => false,

            // auto_now / auto_now_update — temporal types
            (AutoNow, Datetime | Timestamp | TimestampTz) => true,
            (AutoNow, _) => false,
            (AutoNowUpdate, Datetime | Timestamp | TimestampTz) => true,
            (AutoNowUpdate, _) => false,

            // unique — all types except files/bool
            (Unique, Image | Document | File | Bool) => false,
            (Unique, _) => true,

            // fk — any integer/uuid column can reference a PK of the same shape.
            // Uuid is reachable via the `Pk` alias under the `pk-uuid` feature, not
            // just a literal `uuid` field.
            (FormFieldAttr::Fk(_), Int | Bigint | I8 | I16 | U32 | U64 | Uuid) => true,
            (FormFieldAttr::Fk(_), _) => false,

            // skip — all types
            (FormFieldAttr::Skip, _) => true,
        };

        if !valid {
            let attr_name = attr_name_str(attr);
            return Err(syn::Error::new(
                kind_ident.span(),
                format!(
                    "`{}` is not valid for type `{}` (field: {})",
                    attr_name, kind_name, name
                ),
            ));
        }
    }

    // upload_to required for image / document / file
    if matches!(kind, Image | Document | File) && !attrs.iter().any(|a| matches!(a, UploadTo(_))) {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`upload_to` is required for `{}` fields (field: {})",
                kind_name, name
            ),
        ));
    }

    // min < max for int
    let min_val = attrs
        .iter()
        .find_map(|a| if let Min(v) = a { Some(*v) } else { None });
    let max_val = attrs
        .iter()
        .find_map(|a| if let Max(v) = a { Some(*v) } else { None });
    if let (Some(mn), Some(mx)) = (min_val, max_val)
        && mn >= mx
    {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`min` must be less than `max` (field: {}, min={}, max={})",
                name, mn, mx
            ),
        ));
    }

    // min_f < max_f for float/decimal
    let min_f_val = attrs
        .iter()
        .find_map(|a| if let MinF(v) = a { Some(*v) } else { None });
    let max_f_val = attrs
        .iter()
        .find_map(|a| if let MaxF(v) = a { Some(*v) } else { None });
    if let (Some(mn), Some(mx)) = (min_f_val, max_f_val)
        && mn >= mx
    {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "`min` must be less than `max` (field: {}, min={}, max={})",
                name, mn, mx
            ),
        ));
    }

    // `default` literal must roughly match the field's kind — catches copy/paste
    // mistakes like `age: int [default: "abc"]` at parse time instead of a much
    // later, cryptic SeaORM/SQL type error.
    if let Some(Default(lit)) = attrs.iter().find(|a| matches!(a, Default(_))) {
        let matches_kind = match kind {
            Bool => matches!(lit, syn::Lit::Bool(_)),
            Int | Bigint | I8 | I16 | U32 | U64 => matches!(lit, syn::Lit::Int(_)),
            Float | Decimal | Percent | F32 => {
                matches!(lit, syn::Lit::Int(_) | syn::Lit::Float(_))
            }
            _ => matches!(lit, syn::Lit::Str(_)),
        };
        if !matches_kind {
            return Err(syn::Error::new(
                name.span(),
                format!(
                    "`default` value type does not match field type `{}` (field: {})",
                    kind_name, name
                ),
            ));
        }
    }

    Ok(())
}
