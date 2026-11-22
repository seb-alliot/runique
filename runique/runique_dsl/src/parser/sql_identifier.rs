//! Table/identifier name validation shared by the DSL parser.
use syn::{LitStr, Result};

/// Rejects table names that are not a plain SQL identifier (letters/digits/underscore,
/// not starting with a digit). This name is later interpolated unescaped into generated
/// Rust source (migration files, `Alias::new("...")`), so a stray `"` or `;` in the DSL
/// would corrupt or inject into that generated file rather than just failing to compile.
pub(crate) fn validate_sql_identifier(lit: &LitStr) -> Result<()> {
    let value = lit.value();
    let valid = is_sql_identifier(&value);
    if !valid {
        return Err(syn::Error::new(
            lit.span(),
            format!(
                "Invalid table name '{}': must be 1-63 characters, start with a letter or \
                underscore, and contain only letters, digits, and underscores.",
                value
            ),
        ));
    }
    Ok(())
}

fn is_sql_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 63
        && value
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Same rule for the name a `renamed_from` gives. `makemigrations` builds a
/// snapshot path from it and writes it into the migration: anything but a
/// plain identifier would let a model reach a file outside the snapshots or
/// corrupt the generated source.
pub(crate) fn validate_renamed_from(lit: &LitStr) -> Result<()> {
    let value = lit.value();
    if !is_sql_identifier(&value) {
        return Err(syn::Error::new(
            lit.span(),
            format!(
                "Invalid `renamed_from` '{value}': must be 1-63 characters, start with a letter or \
                underscore, and contain only letters, digits, and underscores."
            ),
        ));
    }
    Ok(())
}
