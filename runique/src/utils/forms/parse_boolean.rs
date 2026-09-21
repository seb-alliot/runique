use crate::utils::aliases::StrMap;

/// Reads `key` from submitted form data and interprets it as a checkbox-style
/// boolean: `"on"`, `"true"` or `"1"` is `true`, anything else (including the
/// key being absent, as an unchecked HTML checkbox omits itself) is `false`.
pub fn parse_bool(data: &StrMap, key: &str) -> bool {
    data.get(key)
        .map(|v| v == "on" || v == "true" || v == "1")
        .unwrap_or(false)
}
