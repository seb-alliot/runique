//! Case converter — snake_case to PascalCase, used for code generation.

/// Converts a snake_case string to PascalCase.
///
/// `"user_profile"` → `"UserProfile"`, `"tag"` → `"Tag"`.
pub fn to_pascal_case(words: &str) -> String {
    words
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}
