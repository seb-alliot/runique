pub fn snake_case(name: &str) -> String {
    let mut result = String::new();
    for c in name.chars() {
        if c.is_uppercase() && !result.is_empty() && !result.ends_with('_') {
            result.push('_');
        }
        result.extend(c.to_lowercase());
    }
    result
}
