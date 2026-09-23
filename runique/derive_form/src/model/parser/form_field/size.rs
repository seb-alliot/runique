//! `max_size`/`max_size_mb` value parsing (with optional unit suffix).
use syn::{Ident, LitInt, Result, parse::ParseStream};

/// Parses a size with unit (KB, MB, GB). Returns the value in Bytes.
/// Without unit, the value is treated as MB by default for backward compatibility.
pub(super) fn parse_size(input: ParseStream) -> Result<u64> {
    let n: LitInt = input.parse()?;
    let value = n.base10_parse::<u64>()?;

    if input.peek(Ident) {
        let unit: Ident = input.parse()?;
        let unit_str = unit.to_string().to_uppercase();
        match unit_str.as_str() {
            "KB" | "K" | "KO" => Ok(value * 1024),
            "MB" | "M" | "MO" => Ok(value * 1024 * 1024),
            "GB" | "G" | "GO" => Ok(value * 1024 * 1024 * 1024),
            _ => Err(syn::Error::new(
                unit.span(),
                "Unknown size unit. Expected: KB, MB, GB (or K, M, G, KO, MO, GO)",
            )),
        }
    } else {
        // Default: MB for backward compatibility
        Ok(value * 1024 * 1024)
    }
}
