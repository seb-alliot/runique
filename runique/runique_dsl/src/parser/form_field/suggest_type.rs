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
        "choice",
        "radio",
        "checkbox",
    ];
    // The known type sharing the longest prefix with the input (2 characters at least).
    let common = |k: &str| {
        k.chars()
            .zip(input.chars())
            .take_while(|(a, b)| a == b)
            .count()
    };
    // On a tie, the one whose length is closest to what was typed.
    let closeness = |k: &str| (common(k), std::cmp::Reverse(k.len().abs_diff(input.len())));
    match known.iter().copied().max_by_key(|k| closeness(k)) {
        Some(best) if common(best) >= 2 => format!(" — did you mean `{best}`?"),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::suggest_form_field_type;

    #[test]
    fn suggests_by_common_prefix() {
        assert_eq!(suggest_form_field_type("textt"), " — did you mean `text`?");
        assert_eq!(
            suggest_form_field_type("datetim"),
            " — did you mean `datetime`?"
        );
    }

    #[test]
    fn no_suggestion_without_a_close_name() {
        assert_eq!(suggest_form_field_type("zzz"), "");
        assert_eq!(
            suggest_form_field_type("x"),
            "",
            "one character isn't enough"
        );
    }
}
