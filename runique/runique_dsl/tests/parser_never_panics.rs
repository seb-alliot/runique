//! Property: whatever the input, the DSL parser answers with an AST or a
//! `syn::Error` — it never panics. Inside the `model!{}` / `extend!{}` macros a
//! panic becomes an unreadable compile error, with no position in the
//! developer's code.
//!
//! Two kinds of input: arbitrary text, and sequences of the DSL's own tokens
//! (keywords, types, attributes, punctuation, huge numbers), which reach much
//! deeper into the grammar than random characters do.
use proptest::prelude::*;
use runique_dsl::ast::{EnumDef, FormFieldDecl, MetaDef, ModelInput};
use runique_dsl::extend::ExtendDsl;

/// Runs every parser on `src`; only a panic fails the property.
fn parse_everything(src: &str) {
    // What the macros call on a parsed model must not panic either.
    if let Ok(model) = syn::parse_str::<ModelInput>(src) {
        for field in &model.fields {
            let _ = field.column_type();
            let _ = field.kind.widget();
            let _ = field.kind.unsupported_on();
            let _ = field.kind.byte_limit(None);
        }
    }
    let _ = syn::parse_str::<FormFieldDecl>(src);
    let _ = syn::parse_str::<EnumDef>(src);
    let _ = syn::parse_str::<MetaDef>(src);
    let _ = syn::parse_str::<ExtendDsl>(src);
}

fn dsl_token() -> impl Strategy<Value = String> {
    prop_oneof![
        // Structure
        Just("Model".into()),
        Just("table".into()),
        Just("pk".into()),
        Just("id".into()),
        Just("=>".into()),
        Just("i32".into()),
        Just("Pk".into()),
        Just("enums".into()),
        Just("fields".into()),
        Just("relations".into()),
        Just("meta".into()),
        Just("belongs_to".into()),
        Just("has_many".into()),
        Just("many_to_many".into()),
        Just("via".into()),
        Just("through".into()),
        Just("as".into()),
        Just("ordering".into()),
        Just("unique_together".into()),
        Just("indexes".into()),
        // Types
        Just("text".into()),
        Just("int".into()),
        Just("i8".into()),
        Just("u64".into()),
        Just("decimal".into()),
        Just("choice".into()),
        Just("image".into()),
        Just("binary".into()),
        Just("timestamp_tz".into()),
        Just("password".into()),
        // Attributes
        Just("required".into()),
        Just("nullable".into()),
        Just("max_length".into()),
        Just("min".into()),
        Just("max".into()),
        Just("max_size".into()),
        Just("upload_to".into()),
        Just("default".into()),
        Just("enum".into()),
        Just("fk".into()),
        Just("label".into()),
        Just("cascade".into()),
        Just("KB".into()),
        Just("GB".into()),
        // Punctuation
        Just(":".into()),
        Just(",".into()),
        Just("[".into()),
        Just("]".into()),
        Just("{".into()),
        Just("}".into()),
        Just("(".into()),
        Just(")".into()),
        Just("=".into()),
        Just("-".into()),
        Just(".".into()),
        // Literals, including ones that overflow every integer type
        "[0-9]{1,30}",
        "[0-9]{1,5}(KB|MB|GB|TB|xx)",
        "-?[0-9]{1,25}\\.[0-9]{1,5}",
        "\"[a-zA-Z_/ \\\\\"]{0,12}\"",
        Just("true".into()),
        Just("false".into()),
        "[a-z_]{1,10}",
    ]
}

/// A number of any size, signed or not, sometimes far past every integer type.
fn number() -> impl Strategy<Value = String> {
    prop_oneof![
        "[0-9]{1,3}",
        "[0-9]{10,30}",
        "-[0-9]{1,30}",
        "[0-9]{1,5}\\.[0-9]{1,5}"
    ]
}

/// One attribute, well-formed in shape, with a random value.
fn attribute() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("required".to_string()),
        Just("nullable".to_string()),
        Just("unique".to_string()),
        Just("auto_now".to_string()),
        number().prop_map(|n| format!("max_length: {n}")),
        number().prop_map(|n| format!("min_length: {n}")),
        number().prop_map(|n| format!("min: {n}")),
        number().prop_map(|n| format!("max: {n}")),
        number().prop_map(|n| format!("step: {n}")),
        number().prop_map(|n| format!("rows: {n}")),
        (number(), "(KB|MB|GB|TB|K|go|)").prop_map(|(n, u)| format!("max_size: {n}{u}")),
        (number(), "(KB|MB|GB|TB|K|go)").prop_map(|(n, u)| format!("max_size: {n} {u}")),
        number().prop_map(|n| format!("default: {n}")),
        Just(r#"upload_to: "files/""#.to_string()),
        Just(r#"label: "Label""#.to_string()),
        Just("enum(Status)".to_string()),
        Just("fk(users.id, cascade)".to_string()),
    ]
}

fn field_type() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "text",
        "email",
        "password",
        "int",
        "bigint",
        "i8",
        "i16",
        "u32",
        "u64",
        "float",
        "f32",
        "decimal",
        "percent",
        "bool",
        "date",
        "time",
        "datetime",
        "timestamp",
        "timestamp_tz",
        "uuid",
        "json",
        "image",
        "document",
        "file",
        "choice",
        "radio",
        "checkbox",
        "binary",
        "var_binary",
        "blob",
        "phone",
        "slug",
        "color",
        "ip",
    ])
    .prop_map(str::to_string)
}

/// `name: type [attr, attr, …]`
fn field() -> impl Strategy<Value = String> {
    (
        "[a-z]{1,8}",
        field_type(),
        prop::collection::vec(attribute(), 0..5),
    )
        .prop_map(|(name, ty, attrs)| format!("{name}: {ty} [{}]", attrs.join(", ")))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    #[test]
    fn arbitrary_text_never_panics(src in ".{0,200}") {
        parse_everything(&src);
    }

    #[test]
    fn dsl_shaped_input_never_panics(tokens in prop::collection::vec(dsl_token(), 0..60)) {
        parse_everything(&tokens.join(" "));
    }

    /// Well-formed fields with random attributes and values: reaches the
    /// parsing of each attribute's value (lengths, bounds, sizes, defaults).
    #[test]
    fn random_attribute_values_never_panic(fields in prop::collection::vec(field(), 1..6)) {
        let src = format!(
            r#"Model, table: "items", pk: id => i32, enums: {{ Status: [A, B] }}, {{ {} }}"#,
            fields.join(", ")
        );
        parse_everything(&src);
        for f in &fields {
            parse_everything(f);
        }
    }

    /// A well-formed header, then a random body: reaches the field grammar.
    #[test]
    fn random_fields_never_panic(tokens in prop::collection::vec(dsl_token(), 0..40)) {
        let src = format!(r#"Model, table: "items", pk: id => i32, {{ {} }}"#, tokens.join(" "));
        parse_everything(&src);
    }
}
