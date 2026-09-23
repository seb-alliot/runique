//! `{ name: type [attr1, attr2, ...], ... }` field declaration parsing —
//! the anonymous-block v2 grammar.
use crate::model::ast::{FormFieldAttr, FormFieldDecl, FormFieldKind};
use syn::{
    Ident, LitFloat, LitInt, LitStr, Result, Token,
    parse::{Parse, ParseStream},
    token,
};

use super::size::parse_size;
use super::suggest_type::suggest_form_field_type;
use super::validate_attrs::validate_form_field_attrs;
use crate::model::ast::FkDef;

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

                // `renamed_from: "old"` is a migration-only directive (RENAME COLUMN).
                // It does not affect the generated entity/form, so consume and ignore it here.
                if attr_ident == "renamed_from" {
                    attrs_content.parse::<Token![:]>()?;
                    let _: LitStr = attrs_content.parse()?;
                    let _ = attrs_content.parse::<Token![,]>();
                    continue;
                }

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
                    "step" => {
                        attrs_content.parse::<Token![:]>()?;
                        let n: LitFloat = attrs_content.parse()?;
                        FormFieldAttr::Step(n.base10_parse()?)
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
                        let content;
                        syn::parenthesized!(content in attrs_content);
                        let fk = FkDef::parse(&content)?;
                        FormFieldAttr::Fk(fk)
                    }
                    "skip" => FormFieldAttr::Skip,
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

        let _ = input.parse::<Token![,]>();
        Ok(FormFieldDecl { name, kind, attrs })
    }
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
        ok("age: int [min: 0, max: 150]");
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
        ok("age: int [min: 0, max: 120]");
    }

    // ── min / max (float) ──────────────────────────────────────

    #[test]
    fn min_f_max_f_valid_on_float() {
        ok("score: float [min: 0.0, max: 20.0]");
    }

    #[test]
    fn min_f_max_f_valid_on_decimal() {
        ok("price: decimal [min: 0.0, max: 9999.99]");
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
    fn step_now_valid_on_percent() {
        ok("rate: percent [step: 0.5]");
    }

    #[test]
    fn step_still_invalid_on_int() {
        err("age: int [step: 1.0]");
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
    fn fk_now_valid_on_i8() {
        ok("category_id: i8 [fk(categories.id, cascade)]");
    }

    #[test]
    fn fk_now_valid_on_i16() {
        ok("category_id: i16 [fk(categories.id, cascade)]");
    }

    #[test]
    fn fk_now_valid_on_u32() {
        ok("category_id: u32 [fk(categories.id, cascade)]");
    }

    #[test]
    fn fk_now_valid_on_u64() {
        ok("category_id: u64 [fk(categories.id, cascade)]");
    }

    #[test]
    fn fk_still_invalid_on_text() {
        err("category_id: text [fk(categories.id, cascade)]");
    }
}
