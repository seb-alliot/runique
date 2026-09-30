//! "Did you mean?" suggestion for an unrecognized field type name.

pub(super) fn suggest_form_field_type(input: &str) -> String {
    let known = [
        "text",
        "email",
        "password",
        "richtext",
        "textarea",
        "url",
        "int",
        "bigint",
        "float",
        "decimal",
        "percent",
        "bool",
        "date",
        "time",
        "datetime",
        "image",
        "document",
        "file",
        "color",
        "slug",
        "uuid",
        "json",
        "ip",
        "phone",
        "char",
        "i8",
        "i16",
        "u32",
        "u64",
        "f32",
        "timestamp",
        "timestamp_tz",
        "json_binary",
        "binary",
        "var_binary",
        "blob",
        "cidr",
        "mac_address",
        "interval",
    ];
    // Suggestion by common prefix (≥ 2 characters)
    let matches: Vec<&str> = known
        .iter()
        .filter(|&&k| {
            let min_len = k.len().min(input.len()).min(4);
            min_len >= 2 && k[..min_len] == input[..min_len.min(input.len())]
        })
        .copied()
        .collect();
    if matches.is_empty() {
        String::new()
    } else {
        format!(" — did you mean `{}`?", matches[0])
    }
}
