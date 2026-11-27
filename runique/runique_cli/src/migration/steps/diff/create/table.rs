//! A table created.
use crate::migration::*;

/// The changes that create the table of `schema` from nothing.
pub fn creation_of(schema: &ParsedSchema) -> Changes {
    Changes {
        table_name: schema.table_name.clone(),
        added_columns: db_columns(schema).into_iter().cloned().collect(),
        added_fks: schema.foreign_keys.clone(),
        added_indexes: schema.indexes.clone(),
        table: TableChange::Created,
        ..Default::default()
    }
}
