use super::*;

/// Name and stored values of a string-backed enum column, for the snapshot;
/// `(None, [])` otherwise.
pub(super) fn enum_values(ty: &FieldType, enums: &[EnumDef]) -> (Option<String>, Vec<String>) {
    match ty {
        FieldType::Enum(id) => match find_enum(enums, id) {
            Some(e) if matches!(e.backing_type, EnumBackingType::Auto) => (
                Some(e.name.to_string()),
                e.variants.iter().map(|v| v.db_str()).collect(),
            ),
            _ => (None, Vec::new()),
        },
        _ => (None, Vec::new()),
    }
}
