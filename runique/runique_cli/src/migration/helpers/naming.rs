//! Names: from the PascalCase of a model to the snake_case of its table.

/// Converts a PascalCase string to snake_case.
///
/// # Example
///
/// ```rust
/// use runique_cli::migration::to_snake_case;
/// assert_eq!(to_snake_case("PascalCase"), "pascal_case");
/// assert_eq!(to_snake_case("Test"), "test");
/// ```
pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_ascii_lowercase());
    }
    result
}

/// PascalCase → snake_case to derive target table name from model
pub(crate) fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.extend(ch.to_lowercase());
    }
    result
}
