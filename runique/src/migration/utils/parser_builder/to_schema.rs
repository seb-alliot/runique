//! Conversion from the parsed DSL AST ([`DslModel`]) to the engine-agnostic
//! [`ParsedSchema`] the migration generator consumes.
use super::model::DslModel;
use super::relation::DslRelationKind;
use super::type_mapping::{dsl_field_type_to_col_type, dsl_pk_to_col_type};
use crate::migration::utils::types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};

/// PascalCase → snake_case to derive target table name from model
pub(super) fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

pub(super) fn dsl_to_parsed_schema(model: DslModel) -> ParsedSchema {
    let primary_key = Some(ParsedColumn {
        name: model.pk.name,
        col_type: dsl_pk_to_col_type(&model.pk.ty),
        nullable: false,
        unique: false,
        ignored: false,
        created_at: false,
        updated_at: false,
        has_default_now: false,
        default_value: None,
        enum_name: None,
        enum_string_values: Vec::new(),
        enum_is_pg: false,
        renamed_from: None,
    });

    let enum_types = model.enum_types;

    let columns = model
        .fields
        .into_iter()
        .map(|f| {
            let has_auto_now = f.options.contains(&"auto_now".to_string());
            let has_auto_now_update = f.options.contains(&"auto_now_update".to_string());
            let has_required = f.options.contains(&"required".to_string());
            let has_nullable = f.options.contains(&"nullable".to_string());
            let is_created_at = f.name == "created_at";
            let is_updated_at = f.name == "updated_at";

            // Semantic v2 types (lowercase): lack of `required` → nullable by default.
            // v1 types (SQL / Rust: String, i32...): only explicit `[nullable]` makes it nullable.
            // auto_now / auto_now_update → never nullable in both cases.
            const V2_TYPES: &[&str] = &[
                "text",
                "email",
                "password",
                "richtext",
                "textarea",
                "url",
                "int",
                "bool",
                "boolean",
                "float",
                "decimal",
                "percent",
                "date",
                "time",
                "datetime",
                "timestamp",
                "timestamp_tz",
                "image",
                "document",
                "file",
                "color",
                "slug",
                "uuid",
                "json",
                "json_binary",
                "ip",
                "choice",
                "radio",
                "bigint",
                "binary",
                "blob",
                "inet",
                "cidr",
                "mac_address",
                "interval",
                "phone",
            ];
            let is_v2 = V2_TYPES.contains(&f.ty.as_str());
            let nullable = if has_auto_now || has_auto_now_update || has_required {
                false
            } else if has_nullable {
                true
            } else {
                is_v2 // v2 without required → nullable ; v1 without explicit nullable → not nullable
            };

            let unique = f.options.contains(&"unique".to_string());

            // Resolution of the associated enum (v1: ty=="enum", v2: ty=="choice"/"radio")
            let is_enum_field = f.ty == "enum" || f.ty == "choice" || f.ty == "radio";
            let enum_entry = if is_enum_field {
                f.enum_name
                    .as_deref()
                    .and_then(|n| enum_types.iter().find(|(name, _, _)| name == n))
            } else {
                None
            };

            let col_type = if has_auto_now || has_auto_now_update {
                "DateTime".to_string()
            } else if is_enum_field {
                match enum_entry.map(|(_, bt, _)| bt.as_str()).unwrap_or("Auto") {
                    "i32" => "Integer".to_string(),
                    "i64" => "BigInteger".to_string(),
                    _ => "String".to_string(), // Auto / String → VARCHAR
                }
            } else {
                dsl_field_type_to_col_type(&f.ty)
            };

            // Enum string values for diff (only string-backed enums)
            let (enum_name, enum_string_values, enum_is_pg) = if is_enum_field {
                match enum_entry {
                    Some((name, backing, values)) if backing != "i32" && backing != "i64" => {
                        // Auto → `enum_is_pg = false`, the generator decides via DbKind
                        (Some(name.clone()), values.clone(), false)
                    }
                    _ => (None, Vec::new(), false),
                }
            } else {
                (None, Vec::new(), false)
            };

            let ignored = f.options.contains(&"readonly".to_string()) || f.name == "cache_key";

            ParsedColumn {
                name: f.name,
                col_type,
                nullable,
                unique,
                ignored,
                created_at: has_auto_now || is_created_at,
                updated_at: has_auto_now_update || is_updated_at,
                has_default_now: has_auto_now
                    || has_auto_now_update
                    || is_created_at
                    || is_updated_at,
                default_value: f.default_value,
                enum_name,
                enum_string_values,
                enum_is_pg,
                renamed_from: f.renamed_from,
            }
        })
        .collect();

    let foreign_keys = model
        .relations
        .iter()
        .filter_map(|rel| {
            if let DslRelationKind::BelongsTo {
                from_column,
                on_delete,
                on_update,
            } = &rel.kind
            {
                Some(ParsedFk {
                    from_column: from_column.clone(),
                    to_table: pascal_to_snake(&rel.target),
                    to_column: "id".to_string(),
                    on_delete: on_delete.clone(),
                    on_update: on_update.clone(),
                })
            } else {
                None
            }
        })
        .collect();

    let table = model.table.clone();

    // unique_together → unique indexes
    let mut parsed_indexes: Vec<ParsedIndex> = model
        .unique_together
        .iter()
        .map(|cols| ParsedIndex {
            name: format!("{}_{}_uniq", table, cols.join("_")),
            columns: cols.clone(),
            unique: true,
        })
        .collect();

    // indexes → non-unique indexes
    for cols in &model.indexes {
        parsed_indexes.push(ParsedIndex {
            name: format!("idx_{}_{}", table, cols.join("_")),
            columns: cols.clone(),
            unique: false,
        });
    }

    ParsedSchema {
        table_name: table,
        primary_key,
        columns,
        foreign_keys,
        indexes: parsed_indexes,
    }
}
