use super::*;

/// Parses the `model!{}` invocation of a Rust source file and returns the
/// model's snake_case name together with its [`ParsedSchema`]. `Ok(None)` when
/// the file has no `model!{}`; an error when the file isn't valid Rust, when
/// the model doesn't parse, or when the file declares more than one model.
pub fn parse_schema_from_source(source: &str) -> Result<Option<(String, ParsedSchema)>> {
    Ok(parse_model_from_source(source)?.map(|m| (m.name, m.schema)))
}
