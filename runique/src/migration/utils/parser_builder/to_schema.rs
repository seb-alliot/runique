//! Conversion from the `runique_dsl` AST to the engine-agnostic
//! [`ParsedSchema`] the migration generator consumes.
use quote::ToTokens;
use runique_dsl::ast::{
    EnumBackingType, EnumDef, FieldOption, FieldType, FkAction, FormFieldAttr, FormFieldDecl,
    ModelInput, PkType, RelationDef,
};
use runique_dsl::form_field_to_field_def;

use crate::migration::utils::types::{ParsedColumn, ParsedFk, ParsedIndex, ParsedSchema};

/// PascalCase → snake_case to derive target table name from model
pub(crate) fn pascal_to_snake(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.extend(ch.to_lowercase());
    }
    result
}

fn fk_action(action: FkAction) -> String {
    match action {
        FkAction::NoAction => "NoAction",
        FkAction::Cascade => "Cascade",
        FkAction::SetNull => "SetNull",
        FkAction::Restrict => "Restrict",
        FkAction::SetDefault => "SetDefault",
    }
    .to_string()
}

/// The column type name the migration generator writes for a SQL type.
/// `ip`/`cidr`/`mac_address`/`interval` stay strings: their native types
/// exist only on Postgres.
fn col_type(ty: &FieldType, enums: &[EnumDef]) -> String {
    match ty {
        FieldType::String | FieldType::Varchar(_) | FieldType::Char => "String",
        FieldType::Text => "Text",
        FieldType::I8 => "TinyInteger",
        FieldType::I16 => "SmallInteger",
        FieldType::I32 => "Integer",
        FieldType::I64 => "BigInteger",
        FieldType::U32 => "Unsigned",
        FieldType::U64 => "BigUnsigned",
        FieldType::F32 => "Float",
        FieldType::F64 => "Double",
        FieldType::Decimal(_) => "Decimal",
        FieldType::Bool => "Boolean",
        FieldType::Date => "Date",
        FieldType::Time => "Time",
        FieldType::Datetime | FieldType::Timestamp => "DateTime",
        FieldType::TimestampTz => "TimestampWithTimeZone",
        FieldType::Uuid => "Uuid",
        FieldType::Json | FieldType::JsonBinary => "Json",
        FieldType::Binary(_) => "Binary",
        FieldType::VarBinary(_) => "VarBinary",
        FieldType::Blob => "Blob",
        FieldType::Inet | FieldType::Cidr | FieldType::MacAddress | FieldType::Interval => "String",
        FieldType::Enum(name) => match find_enum(enums, name).map(|e| &e.backing_type) {
            Some(EnumBackingType::I8) => "TinyInteger",
            Some(EnumBackingType::I16) => "SmallInteger",
            Some(EnumBackingType::I32) => "Integer",
            Some(EnumBackingType::I64) => "BigInteger",
            _ => "String",
        },
    }
    .to_string()
}

fn find_enum<'a>(enums: &'a [EnumDef], name: &syn::Ident) -> Option<&'a EnumDef> {
    enums.iter().find(|e| e.name == *name)
}

/// One declared field as a migration column. Shared by `model!{}` and
/// `extend!{}`, which declare fields with the same grammar.
pub(crate) fn decl_to_column(decl: &FormFieldDecl, enums: &[EnumDef]) -> ParsedColumn {
    let field = form_field_to_field_def(decl);
    let has = |wanted: fn(&FieldOption) -> bool| field.options.iter().any(wanted);
    let auto_now = has(|o| matches!(o, FieldOption::AutoNow));
    let auto_now_update = has(|o| matches!(o, FieldOption::AutoNowUpdate));
    let name = field.name.to_string();

    let ty = field.column_type();
    let max_length = match &ty {
        FieldType::Varchar(n) | FieldType::VarBinary(n) => Some(*n),
        FieldType::Binary(n) => field.kind.byte_limit(*n),
        _ => None,
    };
    // Same rule as `ColumnDef::auto_now`: a date-time column, except that a
    // `timestamp_tz` keeps its time zone (its entity field is a `DateTime<Utc>`).
    let col_type = if (auto_now || auto_now_update) && !matches!(ty, FieldType::TimestampTz) {
        "DateTime".to_string()
    } else {
        col_type(&ty, enums)
    };

    // Only string-backed enums carry values (the generator emits CREATE TYPE for them on PG).
    let (enum_name, enum_string_values) = enum_values(&ty, enums);

    let default_value = field.options.iter().find_map(|o| match o {
        FieldOption::Default(lit) => Some(lit.to_token_stream().to_string()),
        _ => None,
    });
    let renamed_from = decl.attrs.iter().find_map(|a| match a {
        FormFieldAttr::RenamedFrom(old) => Some(old.clone()),
        _ => None,
    });

    ParsedColumn {
        col_type,
        nullable: has(|o| matches!(o, FieldOption::Nullable)),
        unique: has(|o| matches!(o, FieldOption::Unique)),
        ignored: has(|o| matches!(o, FieldOption::Readonly)),
        has_default_now: auto_now || auto_now_update,
        default_value,
        enum_name,
        enum_string_values,
        renamed_from,
        max_length,
        name,
    }
}

/// The table of each list field (`checkbox` / `multichoice`): one row per
/// value chosen, `(owner_id, value)` unique, deleted with its owner. Named
/// `{owner table}_{field}`: owner and field together, so two models, or two
/// fields of one model, using the same enum never share it.
pub(super) fn list_tables(model: &ModelInput, owner: &ParsedSchema) -> Vec<ParsedSchema> {
    let Some(owner_pk) = owner.primary_key.as_ref() else {
        return Vec::new();
    };
    model
        .lists
        .iter()
        .map(|decl| {
            let table = format!("{}_{}", owner.table_name, decl.name);
            let enum_ref = decl.attrs.iter().find_map(|a| match a {
                FormFieldAttr::EnumRef(id) => Some(id.clone()),
                _ => None,
            });
            let ty = enum_ref.map_or(FieldType::String, FieldType::Enum);
            let (enum_name, enum_string_values) = enum_values(&ty, &model.enums);
            let column = |name: &str, col_type: String| ParsedColumn {
                name: name.to_string(),
                col_type,
                ..ParsedColumn::default()
            };
            ParsedSchema {
                primary_key: Some(ParsedColumn {
                    name: "id".to_string(),
                    col_type: "BigInteger".to_string(),
                    ..ParsedColumn::default()
                }),
                columns: vec![
                    column("owner_id", owner_pk.col_type.clone()),
                    ParsedColumn {
                        enum_name,
                        enum_string_values,
                        ..column("value", col_type(&ty, &model.enums))
                    },
                ],
                foreign_keys: vec![ParsedFk {
                    from_column: "owner_id".to_string(),
                    to_table: owner.table_name.clone(),
                    to_column: owner_pk.name.clone(),
                    on_delete: fk_action(FkAction::Cascade),
                    on_update: fk_action(FkAction::NoAction),
                }],
                indexes: vec![
                    ParsedIndex {
                        name: format!("{table}_owner_id_value_uniq"),
                        columns: vec!["owner_id".to_string(), "value".to_string()],
                        unique: true,
                    },
                    ParsedIndex {
                        name: format!("idx_{table}_value_owner_id"),
                        columns: vec!["value".to_string(), "owner_id".to_string()],
                        unique: false,
                    },
                ],
                table_name: table,
            }
        })
        .collect()
}

/// Name and stored values of a string-backed enum column, for the snapshot;
/// `(None, [])` otherwise.
fn enum_values(ty: &FieldType, enums: &[EnumDef]) -> (Option<String>, Vec<String>) {
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

pub(super) fn model_to_parsed_schema(model: &ModelInput) -> ParsedSchema {
    let pk_type = match model.pk.ty {
        PkType::I32 => "Integer",
        PkType::I64 => "BigInteger",
        PkType::Uuid => "Uuid",
    };
    let primary_key = Some(ParsedColumn {
        name: model.pk.name.to_string(),
        col_type: pk_type.to_string(),
        ..ParsedColumn::default()
    });

    let columns: Vec<ParsedColumn> = model
        .form_fields
        .iter()
        .map(|decl| decl_to_column(decl, &model.enums))
        .collect();

    let foreign_keys: Vec<ParsedFk> = model
        .relations
        .iter()
        .filter_map(|rel| match rel {
            RelationDef::BelongsTo {
                model: target,
                via,
                on_delete,
                on_update,
            } => Some(ParsedFk {
                from_column: via.to_string(),
                // The target's module name; `scan_entities` swaps in its real
                // table and primary key.
                to_table: pascal_to_snake(&target.to_string()),
                to_column: "id".to_string(),
                on_delete: fk_action(*on_delete),
                on_update: fk_action(*on_update),
            }),
            _ => None,
        })
        .collect();

    let table = model.table.clone();
    let names =
        |cols: &[syn::Ident]| -> Vec<String> { cols.iter().map(|c| c.to_string()).collect() };
    let mut indexes = Vec::new();
    if let Some(meta) = &model.meta {
        for cols in &meta.unique_together {
            let columns = names(cols);
            indexes.push(ParsedIndex {
                name: format!("{}_{}_uniq", table, columns.join("_")),
                columns,
                unique: true,
            });
        }
        for cols in &meta.indexes {
            let columns = names(cols);
            indexes.push(ParsedIndex {
                name: format!("idx_{}_{}", table, columns.join("_")),
                columns,
                unique: false,
            });
        }
    }

    // One index per foreign key column, as Django does: joins and ON DELETE
    // read it, and Postgres/SQLite never create one by themselves (MySQL
    // drops its implicit one once this exists). Skipped when a unique column
    // or a declared index already starts with it.
    for fk in &foreign_keys {
        let covered = columns.iter().any(|c| c.name == fk.from_column && c.unique)
            || indexes
                .iter()
                .any(|i| i.columns.first() == Some(&fk.from_column));
        if !covered {
            indexes.push(ParsedIndex {
                name: format!("idx_{}_{}", table, fk.from_column),
                columns: vec![fk.from_column.clone()],
                unique: false,
            });
        }
    }

    ParsedSchema {
        table_name: table,
        primary_key,
        columns,
        foreign_keys,
        indexes,
    }
}
