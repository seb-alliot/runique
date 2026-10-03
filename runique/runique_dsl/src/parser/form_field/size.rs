//! `max_size`/`max_size_mb` value parsing (with optional unit suffix).
use syn::{Ident, LitInt, Result, parse::ParseStream};

/// Parses a size with unit (KB, MB, GB). Returns the value in Bytes.
/// Without unit, the value is treated as MB by default for backward compatibility.
///
/// The unit can follow the number (`5 MB`) or be glued to it (`500KB`): Rust
/// reads the glued form as one integer literal whose suffix is the unit, so
/// that suffix has to be read too — otherwise `500KB` silently meant 500 MB.
pub(super) fn parse_size(input: ParseStream) -> Result<u64> {
    let n: LitInt = input.parse()?;
    let value = n.base10_parse::<u64>()?;

    let (unit, span) = if !n.suffix().is_empty() {
        (n.suffix().to_uppercase(), n.span())
    } else if input.peek(Ident) {
        let unit: Ident = input.parse()?;
        (unit.to_string().to_uppercase(), unit.span())
    } else {
        // Default: MB for backward compatibility
        ("MB".to_string(), n.span())
    };
    let factor: u64 = match unit.as_str() {
        "KB" | "K" | "KO" => 1024,
        "MB" | "M" | "MO" => 1024 * 1024,
        "GB" | "G" | "GO" => 1024 * 1024 * 1024,
        _ => {
            return Err(syn::Error::new(
                span,
                "Unknown size unit. Expected: KB, MB, GB (or K, M, G, KO, MO, GO)",
            ));
        }
    };
    value
        .checked_mul(factor)
        .ok_or_else(|| syn::Error::new(n.span(), "size too large"))
}

#[cfg(test)]
mod tests {
    use super::parse_size;
    use syn::parse::Parser;

    fn size(src: &str) -> syn::Result<u64> {
        parse_size.parse_str(src)
    }

    #[test]
    fn every_unit_with_or_without_a_space() {
        for (src, bytes) in [
            ("3 KB", 3 * 1024),
            ("3KB", 3 * 1024),
            ("3 k", 3 * 1024),
            ("3ko", 3 * 1024),
            ("3 MB", 3 * 1024 * 1024),
            ("3MB", 3 * 1024 * 1024),
            ("3m", 3 * 1024 * 1024),
            ("3 MO", 3 * 1024 * 1024),
            ("3 GB", 3 * 1024 * 1024 * 1024),
            ("3GB", 3 * 1024 * 1024 * 1024),
            ("3 g", 3 * 1024 * 1024 * 1024),
            ("3GO", 3 * 1024 * 1024 * 1024),
        ] {
            assert_eq!(size(src).unwrap(), bytes, "{src}");
        }
    }

    #[test]
    fn no_unit_means_megabytes() {
        assert_eq!(size("7").unwrap(), 7 * 1024 * 1024);
    }

    #[test]
    fn an_unknown_unit_or_an_overflow_is_an_error_not_a_panic() {
        assert!(size("5 TB").is_err());
        assert!(size("5TB").is_err());
        assert!(size("99999999999999 GB").is_err());
    }
}
