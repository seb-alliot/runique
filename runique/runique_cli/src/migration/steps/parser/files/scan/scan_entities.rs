use super::*;

pub fn scan_entities(entities_path: &str) -> Result<Vec<ParsedSchema>> {
    Ok(scan_models(entities_path)?.schemas)
}
