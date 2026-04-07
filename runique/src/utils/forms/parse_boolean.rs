use crate::utils::aliases::StrMap;

pub fn parse_bool(data: &StrMap, key: &str) -> bool {
    data.get(key)
        .map(|v| v == "on" || v == "true" || v == "1")
        .unwrap_or(false)
}
