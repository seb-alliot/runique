//! `{ name: type [attr1, attr2, ...], ... }` field declaration parsing —
//! the anonymous-block v2 grammar.
use crate::ast::{FormFieldAttr, FormFieldDecl, FormFieldKind};
use syn::{
    Ident, LitFloat, LitInt, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    token,
};

use super::size::parse_size;
use super::suggest_type::suggest_form_field_type;
use super::validate_attrs::validate_form_field_attrs;

impl Parse for FormFieldDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;

        let kind_ident: Ident = input.parse()?;
        let kind = match kind_ident.to_string().as_str() {
            "text" => FormFieldKind::Text,
            "email" => FormFieldKind::Email,
            "password" => FormFieldKind::Password,
            "richtext" => FormFieldKind::Richtext,
            "textarea" => FormFieldKind::Textarea,
            "url" => FormFieldKind::Url,
            "int" => FormFieldKind::Int,
            "float" => FormFieldKind::Float,
            "decimal" => FormFieldKind::Decimal,
            "percent" => FormFieldKind::Percent,
            "bool" => FormFieldKind::Bool,
            "date" => FormFieldKind::Date,
            "time" => FormFieldKind::Time,
            "datetime" => FormFieldKind::Datetime,
            "image" => FormFieldKind::Image,
            "document" => FormFieldKind::Document,
            "file" => FormFieldKind::File,
            "color" => FormFieldKind::Color,
            "slug" => FormFieldKind::Slug,
            "uuid" => FormFieldKind::Uuid,
            "json" => FormFieldKind::Json,
            "ip" => FormFieldKind::Ip,
            "choice" => FormFieldKind::Choice,
            "radio" => FormFieldKind::Radio,
            "checkbox" => FormFieldKind::Checkbox,
            "multichoice" => FormFieldKind::Multichoice,
            "bigint" => FormFieldKind::Bigint,
            "phone" => FormFieldKind::Phone,
            "char" => FormFieldKind::Char,
            "i8" => FormFieldKind::I8,
            "i16" => FormFieldKind::I16,
            "u32" => FormFieldKind::U32,
            "u64" => FormFieldKind::U64,
            "f32" => FormFieldKind::F32,
            "timestamp" => FormFieldKind::Timestamp,
            "timestamp_tz" => FormFieldKind::TimestampTz,
            "json_binary" => FormFieldKind::JsonBinary,
            "binary" => FormFieldKind::Binary,
            "var_binary" => FormFieldKind::VarBinary,
            "blob" => FormFieldKind::Blob,
            "cidr" => FormFieldKind::Cidr,
            "mac_address" => FormFieldKind::MacAddress,
            "interval" => FormFieldKind::Interval,
            // Defers to the global `Pk` alias — resolved immediately to the concrete
            // kind so no downstream consumer needs to know "Pk" was ever written; same
            // principle as the `pk: id => Pk` keyword for the primary key itself.
            "Pk" => {
                #[cfg(feature = "pk-uuid")]
                {
                    FormFieldKind::Uuid
                }
                #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
                {
                    FormFieldKind::Bigint
                }
                #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
                {
                    FormFieldKind::Int
                }
            }
            other => {
                let suggestion = suggest_form_field_type(other);
                return Err(syn::Error::new(
                    kind_ident.span(),
                    format!(
                        "Unknown field type: '{}' (field: {}){}",
                        other, name, suggestion
                    ),
                ));
            }
        };

        // Optional attributes [ ... ]
        let mut attrs = Vec::new();
        if input.peek(token::Bracket) {
            let attrs_content;
            syn::bracketed!(attrs_content in input);
            while !attrs_content.is_empty() {
                // `enum` is a Rust keyword — separate treatment before matching on Ident
                if attrs_content.peek(Token![enum]) {
                    attrs_content.parse::<Token![enum]>()?;
                    let content;
                    syn::parenthesized!(content in attrs_content);
                    let ident: Ident = content.parse()?;
                    attrs.push(FormFieldAttr::EnumRef(ident));
                    let _ = attrs_content.parse::<Token![,]>();
                    continue;
                }

                let attr_ident: Ident = attrs_content.parse()?;

                let attr = match attr_ident.to_string().as_str() {
                    "required" => FormFieldAttr::Required,
                    "nullable" => FormFieldAttr::Nullable,
                    "no_hash" => FormFieldAttr::NoHash,
                    "max_length" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        let val: u32 = n.base10_parse()?;
                        if val == 0 {
                            return Err(syn::Error::new(
                                n.span(),
                                "`max_length` must be greater than 0",
                            ));
                        }
                        FormFieldAttr::MaxLength(val)
                    }
                    "min_length" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        FormFieldAttr::MinLength(n.base10_parse()?)
                    }
                    "min" => {
                        attrs_content.parse::<Token![:]>()?;
                        if attrs_content.peek(LitFloat) {
                            let n: LitFloat = attrs_content.parse()?;
                            FormFieldAttr::MinF(n.base10_parse()?)
                        } else {
                            let n: LitInt = attrs_content.parse()?;
                            FormFieldAttr::Min(n.base10_parse()?)
                        }
                    }
                    "max" => {
                        attrs_content.parse::<Token![:]>()?;
                        if attrs_content.peek(LitFloat) {
                            let n: LitFloat = attrs_content.parse()?;
                            FormFieldAttr::MaxF(n.base10_parse()?)
                        } else {
                            let n: LitInt = attrs_content.parse()?;
                            FormFieldAttr::Max(n.base10_parse()?)
                        }
                    }
                    "default" => {
                        attrs_content.parse::<Token![:]>()?;
                        let lit: syn::Lit = attrs_content.parse()?;
                        FormFieldAttr::Default(lit)
                    }
                    "upload_to" => {
                        attrs_content.parse::<Token![:]>()?;
                        let s: LitStr = attrs_content.parse()?;
                        FormFieldAttr::UploadTo(s.value())
                    }
                    "max_size" | "max_size_mb" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n = parse_size(&attrs_content)?;
                        FormFieldAttr::MaxSize(n)
                    }
                    "rows" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitInt = attrs_content.parse()?;
                        FormFieldAttr::Rows(n.base10_parse()?)
                    }
                    "auto_now" => FormFieldAttr::AutoNow,
                    "auto_now_update" => FormFieldAttr::AutoNowUpdate,
                    "unique" => FormFieldAttr::Unique,
                    "readonly" => FormFieldAttr::Readonly,
                    "label" => {
                        attrs_content.parse::<Token![:]>()?;
                        let s: LitStr = attrs_content.parse()?;
                        FormFieldAttr::Label(s.value())
                    }
                    "fk" => {
                        return Err(syn::Error::new(
                            attr_ident.span(),
                            format!(
                                "`fk(...)` was removed (field: {name}) — declare the relation in `relations: {{ belongs_to: Model via {name} [action] }}`"
                            ),
                        ));
                    }
                    "skip" => FormFieldAttr::Skip,
                    "renamed_from" => {
                        attrs_content.parse::<Token![:]>()?;
                        let s: LitStr = attrs_content.parse()?;
                        FormFieldAttr::RenamedFrom(s.value())
                    }
                    other => {
                        return Err(syn::Error::new(
                            attr_ident.span(),
                            format!("Unknown attribute: '{}' (field: {})", other, name),
                        ));
                    }
                };
                attrs.push(attr);
                let _ = attrs_content.parse::<Token![,]>();
            }
        }

        // Validate attributes vs type
        validate_form_field_attrs(&name, &kind_ident, &kind, &attrs)?;
        let has = |wanted: fn(&FormFieldAttr) -> bool| attrs.iter().any(wanted);
        if has(|a| matches!(a, FormFieldAttr::Required))
            && has(|a| matches!(a, FormFieldAttr::Nullable))
        {
            return Err(syn::Error::new(
                name.span(),
                format!(
                    "field `{name}`: `required` and `nullable` contradict each other — keep one"
                ),
            ));
        }

        if kind.is_list() {
            validate_list(&name, &kind_ident, &attrs)?;
        } else {
            validate_nullability(&name, &kind_ident, &kind, &attrs)?;
        }

        let _ = input.parse::<Token![,]>();
        Ok(FormFieldDecl { name, kind, attrs })
    }
}

/// A list field draws from an enum and has no column of its own: an empty
/// list already says "none", so column attributes don't apply.
fn validate_list(name: &Ident, kind_ident: &Ident, attrs: &[FormFieldAttr]) -> Result<()> {
    if !attrs.iter().any(|a| matches!(a, FormFieldAttr::EnumRef(_))) {
        return Err(syn::Error::new(
            name.span(),
            format!("field `{name}`: `{kind_ident}` needs the enum it draws from — `[enum(Name)]`"),
        ));
    }
    for attr in attrs {
        if !matches!(
            attr,
            FormFieldAttr::Required | FormFieldAttr::EnumRef(_) | FormFieldAttr::Label(_)
        ) {
            return Err(syn::Error::new(
                name.span(),
                format!(
                    "field `{name}`: `{}` isn't valid on a `{kind_ident}` list — only `required`, `enum(...)` and `label` are",
                    super::attr_name::attr_name_str(attr)
                ),
            ));
        }
    }
    Ok(())
}

/// Types whose form field always submits something storable in a NOT NULL
/// column: an empty text, no file, no bytes, an unchecked box (`false`).
fn has_empty_value(kind: &FormFieldKind) -> bool {
    use FormFieldKind::*;
    matches!(
        kind,
        Text | Email
            | Password
            | Richtext
            | Textarea
            | Url
            | Color
            | Slug
            | Phone
            | Char
            | Image
            | Document
            | File
            | Binary
            | VarBinary
            | Blob
            | Bool
    )
}

/// A column is NOT NULL unless declared `nullable`. Refuses `nullable` on a
/// column the framework always fills, and a NOT NULL column whose optional
/// form field can come back empty with nothing to store.
fn validate_nullability(
    name: &Ident,
    kind_ident: &Ident,
    kind: &FormFieldKind,
    attrs: &[FormFieldAttr],
) -> Result<()> {
    let has = |wanted: fn(&FormFieldAttr) -> bool| attrs.iter().any(wanted);
    let nullable = has(|a| matches!(a, FormFieldAttr::Nullable));
    let auto = has(|a| matches!(a, FormFieldAttr::AutoNow | FormFieldAttr::AutoNowUpdate));

    if nullable && auto {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}`: `auto_now`/`auto_now_update` always fill the column — it can't be `nullable`"
            ),
        ));
    }

    let filled = nullable
        || auto
        || has(|a| {
            matches!(
                a,
                FormFieldAttr::Required
                    | FormFieldAttr::Default(_)
                    | FormFieldAttr::Skip
                    | FormFieldAttr::Readonly
            )
        });
    if !filled && !has_empty_value(kind) {
        return Err(syn::Error::new(
            name.span(),
            format!(
                "field `{name}` (`{kind_ident}`): the column is NOT NULL, but an empty optional field has no `{kind_ident}` value to store — add `required`, `nullable` or `default: …`"
            ),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Parses a `FormFieldDecl` from a DSL string.
    /// Format: `name: type [attr1, attr2, ...]`
    fn parse_field(src: &str) -> syn::Result<FormFieldDecl> {
        syn::parse_str::<FormFieldDecl>(src)
    }

    fn ok(src: &str) {
        assert!(
            parse_field(src).is_ok(),
            "expected OK but error for: `{src}`"
        );
    }

    fn err(src: &str) {
        assert!(
            parse_field(src).is_err(),
            "expected ERROR but OK for: `{src}`"
        );
    }

    // ── list fields ───────────────────────────────────────────────

    #[test]
    fn list_fields_draw_from_an_enum() {
        ok("tags: checkbox [enum(Tag)]");
        ok("tags: multichoice [enum(Tag), required, label: \"Tags\"]");
        err("tags: checkbox");
        err("tags: multichoice [required]");
    }

    #[test]
    fn list_fields_refuse_column_attributes() {
        for attr in ["nullable", "unique", "default: \"a\"", "readonly", "skip"] {
            err(&format!("tags: checkbox [enum(Tag), {attr}]"));
        }
    }

    // ── nullability ───────────────────────────────────────────────

    #[test]
    fn a_column_without_an_empty_value_must_say_how_it_is_filled() {
        err("age: int");
        err("born: date");
        err("status: choice [enum(Status)]");
        err("meta: json");
        ok("age: int [required]");
        ok("age: int [nullable]");
        ok("age: int [default: 0]");
        ok("age: int [skip]");
        ok("age: int [readonly]");
    }

    #[test]
    fn a_column_with_an_empty_value_may_stay_optional() {
        ok("name: text");
        ok("bio: textarea");
        ok("active: bool");
        ok(r#"avatar: image [upload_to: "a/"]"#);
        ok("raw: blob");
    }

    #[test]
    fn auto_filled_columns_refuse_nullable() {
        err("created: datetime [auto_now, nullable]");
        err("updated: datetime [auto_now_update, nullable]");
        ok("created: datetime [auto_now]");
    }

    // ── max_length ────────────────────────────────────────────────

    #[test]
    fn max_length_valid_on_text() {
        ok("name: text [max_length: 100]");
    }

    #[test]
    fn max_length_valid_on_email() {
        ok("email: email [max_length: 254]");
    }

    #[test]
    fn max_length_valid_on_password() {
        ok("pwd: password [max_length: 128]");
    }

    #[test]
    fn max_length_valid_on_url() {
        ok("site: url [max_length: 200]");
    }

    #[test]
    fn max_length_invalid_on_int() {
        err("age: int [max_length: 50]");
    }

    #[test]
    fn max_length_invalid_on_float() {
        err("price: float [max_length: 10]");
    }

    #[test]
    fn max_length_invalid_on_bool() {
        err("active: bool [max_length: 5]");
    }

    #[test]
    fn max_length_invalid_on_date() {
        err("birth: date [max_length: 10]");
    }

    #[test]
    fn max_length_invalid_on_uuid() {
        err("uid: uuid [max_length: 36]");
    }

    // ── min / max (integer) ────────────────────────────────────────

    #[test]
    fn min_max_valid_on_int() {
        ok("age: int [required, min: 0, max: 150]");
    }

    #[test]
    fn min_invalid_on_text() {
        err("name: text [min: 0]");
    }

    #[test]
    fn max_invalid_on_float() {
        // integer max (i64) is not valid on float — must use floating max
        err("price: float [max: 100]");
    }

    #[test]
    fn min_invalid_on_bool() {
        err("flag: bool [min: 0]");
    }

    #[test]
    fn min_invalid_on_date() {
        err("day: date [min: 0]");
    }

    #[test]
    fn min_equal_max_rejected() {
        err("age: int [min: 5, max: 5]");
    }

    #[test]
    fn min_greater_than_max_rejected() {
        err("age: int [min: 10, max: 5]");
    }

    #[test]
    fn min_less_than_max_accepted() {
        ok("age: int [required, min: 0, max: 120]");
    }

    // ── min / max (float) ──────────────────────────────────────

    #[test]
    fn min_f_max_f_valid_on_float() {
        ok("score: float [required, min: 0.0, max: 20.0]");
    }

    #[test]
    fn min_f_max_f_valid_on_decimal() {
        ok("price: decimal [required, min: 0.0, max: 9999.99]");
    }

    #[test]
    fn min_f_invalid_on_int() {
        // float min on an int field -> invalid
        err("age: int [min: 0.0]");
    }

    #[test]
    fn min_f_equal_max_f_rejected() {
        err("score: float [min: 5.0, max: 5.0]");
    }

    // ── upload_to ─────────────────────────────────────────────────

    #[test]
    fn upload_to_valid_on_image() {
        ok(r#"avatar: image [upload_to: "avatars/"]"#);
    }

    #[test]
    fn upload_to_valid_on_document() {
        ok(r#"cv: document [upload_to: "docs/"]"#);
    }

    #[test]
    fn upload_to_valid_on_file() {
        ok(r#"piece: file [upload_to: "files/"]"#);
    }

    #[test]
    fn upload_to_required_on_image_without_it() {
        err("avatar: image []");
    }

    #[test]
    fn upload_to_required_on_document_without_it() {
        err("cv: document []");
    }

    #[test]
    fn upload_to_invalid_on_text() {
        err(r#"name: text [upload_to: "path/"]"#);
    }

    #[test]
    fn upload_to_invalid_on_int() {
        err(r#"age: int [upload_to: "path/"]"#);
    }

    #[test]
    fn upload_to_invalid_on_email() {
        err(r#"mail: email [upload_to: "path/"]"#);
    }

    // ── general cases ──────────────────────────────────────────────

    #[test]
    fn field_without_attrs() {
        ok("name: text");
    }

    #[test]
    fn required_universal() {
        ok("age: int [required]");
        ok("name: text [required]");
        ok("flag: bool [required]");
    }

    #[test]
    fn nullable_universal() {
        ok("age: int [nullable]");
        ok("name: text [nullable]");
    }

    // ── field-level regressions fixed 2026-09-19 ────────────────────

    #[test]
    fn max_length_zero_rejected() {
        err("name: text [max_length: 0]");
    }

    #[test]
    fn max_length_nonzero_still_accepted() {
        ok("name: text [max_length: 1]");
    }

    #[test]
    fn default_str_on_text_accepted() {
        ok(r#"name: text [default: "hello"]"#);
    }

    #[test]
    fn default_str_on_int_rejected() {
        err(r#"age: int [default: "abc"]"#);
    }

    #[test]
    fn default_int_on_int_accepted() {
        ok("age: int [default: 0]");
    }

    #[test]
    fn default_bool_on_int_rejected() {
        err("age: int [default: true]");
    }

    #[test]
    fn default_bool_on_bool_accepted() {
        ok("active: bool [default: true]");
    }

    #[test]
    fn default_str_on_bool_rejected() {
        err(r#"active: bool [default: "true"]"#);
    }

    #[test]
    fn default_int_on_float_accepted() {
        // an integer literal is a valid default for a float/decimal field
        ok("price: decimal [default: 0]");
    }

    #[test]
    fn default_float_on_float_accepted() {
        ok("price: decimal [default: 0.59]");
    }

    #[test]
    fn min_length_now_valid_on_email() {
        ok("mail: email [min_length: 5]");
    }

    #[test]
    fn min_length_now_valid_on_password() {
        ok("pwd: password [min_length: 8]");
    }

    #[test]
    fn min_length_now_valid_on_richtext() {
        ok("body: richtext [min_length: 10]");
    }

    #[test]
    fn min_length_now_valid_on_url() {
        ok("site: url [min_length: 5]");
    }

    #[test]
    fn min_length_now_valid_on_binary() {
        ok("blob: binary [min_length: 1]");
    }

    #[test]
    fn min_length_still_invalid_on_int() {
        err("age: int [min_length: 1]");
    }

    #[test]
    fn step_is_not_an_attribute() {
        err("rate: percent [required, step: 0.5]");
    }

    #[test]
    fn auto_now_now_valid_on_timestamp() {
        ok("created_at: timestamp [auto_now]");
    }

    #[test]
    fn auto_now_now_valid_on_timestamp_tz() {
        ok("created_at: timestamp_tz [auto_now]");
    }

    #[test]
    fn auto_now_update_now_valid_on_timestamp() {
        ok("updated_at: timestamp [auto_now_update]");
    }

    #[test]
    fn auto_now_still_invalid_on_date() {
        err("created_at: date [auto_now]");
    }

    #[test]
    fn fk_attribute_points_to_belongs_to() {
        let Err(err) = parse_field("category_id: int [required, fk(categories.id, cascade)]")
        else {
            panic!("fk(...) is refused");
        };
        assert!(err.to_string().contains("belongs_to"), "{err}");
    }

    #[test]
    fn a_refused_attribute_is_named_in_the_error() {
        let Err(err) = parse_field("flag: bool [max_length: 3]") else {
            panic!("max_length is refused on bool");
        };
        let err = err.to_string();
        assert!(err.contains("max_length"), "{err}");
    }

    #[test]
    fn required_and_nullable_together_are_refused() {
        err("name: text [required, nullable]");
        ok("name: text [required]");
        ok("name: text [nullable]");
    }
}
