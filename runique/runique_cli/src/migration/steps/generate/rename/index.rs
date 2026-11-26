//! An index renamed: no engine renames one the same way, so the index is
//! created under its new name, then dropped under the old one. In that order:
//! MySQL refuses to drop the last index a foreign key relies on.
use crate::migration::steps::generate::{create, delete};
use crate::migration::types::ParsedIndex;

/// `from` and `to`: the index, and the table name it was named for (see
/// `create::index`).
pub(crate) fn index(table: &str, from: (&ParsedIndex, &str), to: (&ParsedIndex, &str)) -> String {
    let mut out = create::index(table, Some(to.1), to.0);
    out.push_str(&delete::index(table, Some(from.1), from.0));
    out
}
