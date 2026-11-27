use super::*;

/// What the files of a run are for. A stage has no place of its own:
/// `Order::of` gives it one, the same order the instructions of a file run in.
#[derive(Clone, Copy)]
pub(crate) enum Stage {
    /// Enums that don't exist yet: before the tables, which name them.
    CreateEnums,
    /// Enums that take another name: before the tables, which name them as
    /// the models do.
    RenameEnums,
    /// Tables created or changed, models then `extend!{}` blocks.
    Tables,
    /// Values renamed or added in an enum: once every table has its new
    /// names.
    EnumValues,
    /// Tables dropped: what destroys closes the run, once the keys of the
    /// tables that stay were removed by their own migrations.
    DropTables,
}

impl Order {
    /// Where the files of `stage` run among the others.
    pub(crate) fn of(stage: Stage) -> Order {
        match stage {
            Stage::CreateEnums => Order::CreateEnumTypes,
            Stage::RenameEnums => Order::RenameEnumTypes,
            Stage::Tables => Order::CreateTable,
            Stage::EnumValues => Order::RenameEnumValues,
            Stage::DropTables => Order::DeleteTable,
        }
    }
}
