//! Cross-checks a `FormFieldDecl`'s parsed attributes against its kind
//! (e.g. `max_length` only makes sense on textual types).
use crate::ast::{FormFieldAttr, FormFieldKind};
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
                FormFieldKind::Choice
                | FormFieldKind::Radio
                | FormFieldKind::Checkbox
                | FormFieldKind::Multichoice,
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

            // max_digits / decimal_places — decimal only
            (MaxDigits(_), Decimal) => true,
            (MaxDigits(_), _) => false,
            (DecimalPlaces(_), Decimal) => true,
            (DecimalPlaces(_), _) => false,

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

            // auto_now / auto_now_update — temporal types
            (AutoNow, Datetime | Timestamp | TimestampTz) => true,
            (AutoNow, _) => false,
            (AutoNowUpdate, Datetime | Timestamp | TimestampTz) => true,
            (AutoNowUpdate, _) => false,

            // unique — all types except files/bool
            (Unique, Image | Document | File | Bool) => false,
            (Unique, _) => true,

            // skip / renamed_from — all types
            (FormFieldAttr::Skip, _) => true,
            (RenamedFrom(_), _) => true,
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

    if matches!(kind, Decimal) {
        validate_decimal_len(name, attrs)?;
    }

    // `required` on a box means "must be checked": checked in advance, it would
    // be met without the user doing anything — consent has to be a gesture.
    if matches!(kind, Bool)
        && attrs.iter().any(|a| matches!(a, Required))
        && attrs
            .iter()
            .any(|a| matches!(a, Default(syn::Lit::Bool(checked)) if checked.value))
    {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: `required` on a `bool` means the box must be checked; `default: true` would check it for the user — drop one of them"
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

/// Largest precision and scale MySQL/MariaDB accept, the lowest of the three
/// engines: a model valid here creates the same column everywhere.
const MAX_DIGITS: u32 = 65;
const MAX_DECIMAL_PLACES: u32 = 30;

/// A decimal states both its precision and its scale: left out, MySQL/MariaDB
/// create `DECIMAL(10, 0)` and round every value to an integer.
fn validate_decimal_len(name: &Ident, attrs: &[FormFieldAttr]) -> Result<()> {
    use FormFieldAttr::*;

    let max_digits = attrs.iter().find_map(|a| match a {
        MaxDigits(n) => Some(*n),
        _ => None,
    });
    let decimal_places = attrs.iter().find_map(|a| match a {
        DecimalPlaces(n) => Some(*n),
        _ => None,
    });
    let (Some(max_digits), Some(decimal_places)) = (max_digits, decimal_places) else {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: a `decimal` needs `max_digits` (all its digits) and `decimal_places` (those after the point), e.g. `[max_digits: 10, decimal_places: 2]` for 99999999.99"
            ),
        ));
    };
    if !(1..=MAX_DIGITS).contains(&max_digits) {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: `max_digits` must be between 1 and {MAX_DIGITS} (got {max_digits})"
            ),
        ));
    }
    if decimal_places > MAX_DECIMAL_PLACES {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: `decimal_places` must be at most {MAX_DECIMAL_PLACES} (got {decimal_places})"
            ),
        ));
    }
    if decimal_places > max_digits {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: `decimal_places` ({decimal_places}) can't exceed `max_digits` ({max_digits}), which counts them"
            ),
        ));
    }
    Ok(())
}
