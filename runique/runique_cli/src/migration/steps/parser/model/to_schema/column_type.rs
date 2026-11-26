use super::*;

/// The column a field is stored in: an enum follows the type its values
/// are declared with.
pub(super) fn col_type(ty: &FieldType, enums: &[EnumDef]) -> ColumnType {
    let backing = match ty {
        FieldType::Enum(name) => find_enum(enums, name).map(|e| &e.backing_type),
        _ => None,
    };
    ColumnType::of_field(ty, backing)
}
