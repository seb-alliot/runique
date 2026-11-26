use super::*;

/// Scans all `.rs` files in the `entities_path` directory and collects
/// all found `extend!{}` blocks. Returns a flat list of `ParsedSchema`
/// (one per block — multiple blocks can target the same table).
pub fn scan_extend_blocks(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    let mut schemas = Vec::new();
    for path in rust_files(Path::new(entities_path))? {
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        let source = fs::read_to_string(&path)
            .with_context(|| format!("Cannot read file: {}", path.display()))?;

        let blocks = parse_extend_blocks_from_source(&source)
            .with_context(|| format!("Invalid extend!{{}} in {}", path.display()))?;
        for schema in blocks {
            schemas.push(schema);
        }
    }
    Ok(schemas)
}
