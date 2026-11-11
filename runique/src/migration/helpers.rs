//! Shared helpers: snake_case conversion.

// ============================================================
// String helpers
// ============================================================

/// Converts a PascalCase string to snake_case.
///
/// # Example
///
/// ```rust
/// use runique::migration::to_snake_case;
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
