use super::*;

/// The table renames `model` declares: see [`ParsedModel::renames`].
pub(super) fn declared_renames(model: &ModelInput) -> Vec<(String, String)> {
    let former_table = model.renamed_from.as_deref().unwrap_or(&model.table);
    let mut renames: Vec<(String, String)> = Vec::new();
    if former_table != model.table {
        renames.push((former_table.to_string(), model.table.clone()));
    }
    for decl in &model.lists {
        let name = decl.name.to_string();
        let former_field = decl
            .attrs
            .iter()
            .find_map(|a| match a {
                FormFieldAttr::RenamedFrom(old) => Some(old.as_str()),
                _ => None,
            })
            .unwrap_or(&name);
        let former = format!("{former_table}_{former_field}");
        let now = format!("{}_{}", model.table, name);
        if former != now {
            renames.push((former, now));
        }
    }
    renames
}
