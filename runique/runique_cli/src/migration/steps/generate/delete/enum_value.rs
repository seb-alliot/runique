//! A value removed from an enum: not migrated yet, the file says so.
pub(crate) fn enum_value(enum_name: &str, value: &str) -> String {
    format!(
        "        // WARNING: value {val:?} removed from {enum_name:?} — manual migration required (Postgres cannot drop an enum value).\n\n",
        val = value,
        enum_name = enum_name,
    )
}
