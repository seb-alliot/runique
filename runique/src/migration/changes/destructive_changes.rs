//! Destructive changes: listing them (dropped columns, type changes, narrowed
//! lengths, dropped FKs…) and the type changes that are not migrated yet.
use crate::migration::types::Changes;

/// Scans a set of changes for destructive operations (dropped columns, type
/// changes, nullable-to-required, dropped FKs, new `ON DELETE CASCADE` on an
/// existing table) and returns one human-readable warning per finding. An
/// empty result means the changes are safe to apply without `--force`.
pub fn collect_destructive_messages(all_changes: &[Changes]) -> Vec<String> {
    let dropped = all_changes.iter().flat_map(|c| {
        c.dropped_columns
            .iter()
            .map(|col| format!("  {}.{}: DROP COLUMN (data loss)", c.table_name, col.name))
    });

    let type_changes = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            // `col_type` alone misses becoming (or stopping being) an enum — an enum
            // column keeps col_type "String", the shape change lives in
            // `enum_string_values` instead. Without this, a string->enum column would
            // generate silently (no `--force` prompt) with only a code comment warning
            // the developer might never see.
            .filter(|(old, new)| {
                old.col_type != new.col_type
                    || old.enum_string_values.is_empty() != new.enum_string_values.is_empty()
            })
            .map(|(old, new)| {
                if old.col_type != new.col_type {
                    format!(
                        "  {}.{}: type {} -> {}",
                        c.table_name, old.name, old.col_type, new.col_type
                    )
                } else if new.enum_string_values.is_empty() {
                    format!(
                        "  {}.{}: enum -> {} (manual migration required)",
                        c.table_name, old.name, new.col_type
                    )
                } else {
                    format!(
                        "  {}.{}: {} -> enum (manual migration required)",
                        c.table_name, old.name, old.col_type
                    )
                }
            })
    });

    let nullable_to_required = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| old.nullable && !new.nullable && old.col_type == new.col_type)
            .map(|(_, new)| {
                format!(
                    "  {}.{}: nullable -> not_null (requires a default or backfill)",
                    c.table_name, new.name
                )
            })
    });

    let length_shrinks = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| {
                old.col_type == new.col_type && length_may_shrink(old.max_length, new.max_length)
            })
            .map(|(old, new)| {
                let len =
                    |l: Option<u32>| l.map_or_else(|| "default".to_string(), |n| n.to_string());
                format!(
                    "  {}.{}: length {} -> {} (longer values would be cut or refused)",
                    c.table_name,
                    new.name,
                    len(old.max_length),
                    len(new.max_length)
                )
            })
    });

    let decimal_shrinks = all_changes.iter().flat_map(|c| {
        c.modified_columns
            .iter()
            .filter(|(old, new)| {
                old.col_type == new.col_type
                    && decimal_may_shrink(old.decimal_len, new.decimal_len)
            })
            .map(|(old, new)| {
                let len = |l: Option<(u32, u32)>| {
                    l.map_or_else(|| "default".to_string(), |(p, s)| format!("({p}, {s})"))
                };
                format!(
                    "  {}.{}: decimal {} -> {} (values with more digits would be rounded or refused)",
                    c.table_name,
                    new.name,
                    len(old.decimal_len),
                    len(new.decimal_len)
                )
            })
    });

    let dropped_fks = all_changes.iter().flat_map(|c| {
        c.dropped_fks.iter().map(|fk| {
            format!(
                "  {}.{}: DROP FOREIGN KEY -> {} (orphan records possible)",
                c.table_name, fk.from_column, fk.to_table
            )
        })
    });

    // Adding a CASCADE constraint to existing data can trigger mass deletes if a parent is removed.
    // Only a concern on existing tables: a table created in this same batch has no rows yet,
    // so a CASCADE FK on a brand-new table carries no data-loss risk (avoids a false positive
    // on first generation / from-scratch regeneration).
    let cascade_fks = all_changes
        .iter()
        .filter(|c| !c.is_new_table)
        .flat_map(|c| {
            c.added_fks
                .iter()
                .filter(|fk| fk.on_delete.to_uppercase() == "CASCADE")
                .map(|fk| {
                    format!(
                        "  {}.{}: ADD FOREIGN KEY -> {} ON DELETE CASCADE (existing rows may be deleted)",
                        c.table_name, fk.from_column, fk.to_table
                    )
                })
        });

    dropped
        .chain(type_changes)
        .chain(nullable_to_required)
        .chain(length_shrinks)
        .chain(decimal_shrinks)
        .chain(dropped_fks)
        .chain(cascade_fks)
        .collect()
}

/// Whether going from `old` to `new` column length can drop data on some
/// engine. No length is unbounded on Postgres but `VARCHAR(255)` on MySQL, so
/// both directions out of "no length" are checked against each.
pub fn length_may_shrink(old: Option<u32>, new: Option<u32>) -> bool {
    match (old, new) {
        (Some(old), Some(new)) => new < old,
        (None, Some(_)) => true,
        (Some(old), None) => old > 255,
        (None, None) => false,
    }
}

/// Whether going from `old` to `new` decimal precision can drop data. No
/// precision is unbounded on Postgres and `DECIMAL(10, 0)` on MySQL, so a
/// first precision always may: fewer digits before the point, or after it.
pub fn decimal_may_shrink(old: Option<(u32, u32)>, new: Option<(u32, u32)>) -> bool {
    match (old, new) {
        (Some((old_p, old_s)), Some((new_p, new_s))) => {
            new_s < old_s || new_p.saturating_sub(new_s) < old_p.saturating_sub(old_s)
        }
        (None, Some(_)) => true,
        (_, None) => false,
    }
}

/// The columns whose type changed: `--force` let them through, but their
/// migration holds a warning comment only while the snapshot records the new
/// type, so the database keeps the old one. A change to or from an enum is
/// migrated (`build_enum_transition`) and isn't listed.
pub fn unmigrated_type_changes(changes: &[Changes]) -> Vec<String> {
    changes
        .iter()
        .flat_map(|c| {
            c.modified_columns
                .iter()
                .filter(|(old, new)| {
                    old.col_type != new.col_type
                        && old.enum_string_values.is_empty() == new.enum_string_values.is_empty()
                })
                .map(|(old, new)| {
                    format!(
                        "  {}.{}: {} -> {}",
                        c.table_name, old.name, old.col_type, new.col_type
                    )
                })
        })
        .collect()
}
