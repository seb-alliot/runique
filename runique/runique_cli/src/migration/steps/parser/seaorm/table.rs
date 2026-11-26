use super::*;

/// Parses a generated SeaORM migration snapshot's `up()` function and rebuilds
/// its [`ParsedSchema`] (table name, primary key, columns, FKs, indexes) by
/// walking the method-call chains passed to `.table()`, `.col()`, `.foreign_key()`,
/// `.create_foreign_key()`, `.index()` and `.create_index()`. Fails only if no
/// table name could be extracted.
pub fn parse_seaorm_source(source: &str) -> Result<ParsedSchema> {
    let file =
        syn::parse_str::<syn::File>(source).map_err(|e| anyhow::anyhow!("Parse error: {}", e))?;
    let mut visitor = SeaOrmVisitor::new();
    visitor.visit_file(&file);
    let table_name = visitor
        .table_name
        .ok_or_else(|| anyhow::anyhow!("Cannot extract table name"))?;
    // The columns of a table are written as one chain, `.col(a).col(b)`,
    // whose outermost call is the last one: the visitor meets them last
    // first. Put back in the order of the file, the one of the model.
    let mut columns = visitor.columns;
    columns.reverse();
    Ok(ParsedSchema {
        table_name,
        primary_key: visitor.primary_key,
        columns,
        foreign_keys: visitor.foreign_keys,
        indexes: visitor.indexes,
    })
}
