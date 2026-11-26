use super::*;

/// Same as [`parse_schema_from_source`], with the list fields' tables.
pub fn parse_model_from_source(source: &str) -> Result<Option<ParsedModel>> {
    let file = syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!(located(&e)))?;
    let mut collector = MacroCollector::<ModelInput>::new("model");
    collector.visit_file(&file);

    let mut models = Vec::new();
    for parsed in collector.found {
        models.push(parsed.map_err(|e| anyhow::anyhow!(located(&e)))?);
    }
    if models.len() > 1 {
        bail!(
            "{} `model!{{}}` declarations in one file — one model per file",
            models.len()
        );
    }
    Ok(models.pop().map(|model| {
        let schema = model_to_parsed_schema(&model);
        ParsedModel {
            name: snake_case(&model.name.to_string()),
            lists: list_tables(&model, &schema),
            managed: model.meta.as_ref().is_none_or(|meta| meta.managed),
            renames: declared_renames(&model),
            schema,
        }
    }))
}
