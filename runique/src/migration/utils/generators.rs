//! SeaORM migration Rust code generation — `up`/`down` files, CREATE TABLE, FK, indexes, triggers.
use crate::migration::utils::{
    helpers::col_type_to_method,
    types::{Changes, DbKind, ParsedColumn, ParsedSchema},
};

pub fn generate_create_file(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    let cols = build_create_table_cols(schema, db_kind);
    let fk_stmts = build_fk_create_stmts(schema);
    let idx_stmts = build_index_create_stmts(schema);
    let trigger_stmts = build_updated_at_trigger_stmts(schema, db_kind);
    let enum_stmts = build_enum_type_stmts(schema, db_kind);
    let enum_drops = build_enum_type_drops(schema, db_kind);
    let fk_drops = build_fk_drop_stmts(schema);
    let idx_drops = build_index_drop_stmts(schema);
    let trigger_drops = build_updated_at_trigger_drops(schema, db_kind);

    let mut up = String::new();
    up.push_str(&enum_stmts);
    up.push_str("        manager\n");
    up.push_str("            .create_table(\n");
    up.push_str("                Table::create()\n");
    up.push_str(&format!(
        "                    .table(Alias::new(\"{}\"))\n",
        schema.table_name
    ));
    up.push_str("                    .if_not_exists()\n");
    up.push_str(&cols);
    up.push_str("                    .to_owned()\n"); //
    up.push_str("            )\n");
    up.push_str("            .await?;\n\n");
    up.push_str(&fk_stmts);
    up.push_str(&idx_stmts);
    up.push_str(&trigger_stmts);
    up.push_str("        Ok(())\n");

    let mut down = String::new();
    down.push_str(&trigger_drops);
    down.push_str(&fk_drops);
    down.push_str(&idx_drops);
    down.push_str("        manager\n");
    down.push_str("            .drop_table(Table::drop()\n");
    down.push_str(&format!(
        "                .table(Alias::new(\"{}\"))\n",
        schema.table_name
    ));
    down.push_str("                .to_owned())\n");
    down.push_str("            .await?;\n");
    down.push_str(&enum_drops);
    down.push_str("        Ok(())\n");

    format!(
        "use sea_orm_migration::prelude::*;\n\n\
    #[derive(DeriveMigrationName)]\n\
    pub struct Migration;\n\n\
    #[async_trait::async_trait]\n\
    impl MigrationTrait for Migration {{\n\
        async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {up}\
        }}\n\n\
        async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n\
    {down}\
        }}\n\
    }}\n",
        up = up,
        down = down,
    )
}

fn build_enum_type_stmts(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    if *db_kind != DbKind::Postgres {
        return String::new();
    }
    let mut out = String::new();
    for col in &schema.columns {
        if col.enum_string_values.is_empty() {
            continue;
        }
        let name = col.enum_name.as_deref().unwrap_or(&col.name);
        let variants: Vec<String> = col
            .enum_string_values
            .iter()
            .map(|v| format!("'{}'", v)) // ← single quotes
            .collect();
        out.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"DO $$ BEGIN CREATE TYPE {name} AS ENUM ({variants}); EXCEPTION WHEN duplicate_object THEN NULL; END $$\"\n        ).await?;\n\n",
            name = name,
            variants = variants.join(", "),
        ));
    }
    out
}

fn build_enum_type_drops(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    if *db_kind != DbKind::Postgres {
        return String::new();
    }
    let mut out = String::new();
    for col in &schema.columns {
        if col.enum_string_values.is_empty() {
            continue;
        }
        let name = col.enum_name.as_deref().unwrap_or(&col.name);
        out.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"DROP TYPE IF EXISTS {name}\"\n        ).await?;\n\n",
            name = name,
        ));
    }
    out
}

pub fn generate_alter_file(change: &Changes) -> String {
    let (up_body, down_body) = build_alter_bodies(change);

    let up_param = if up_body.contains("manager.") {
        "manager"
    } else {
        "_manager"
    };
    let down_param = if down_body.contains("manager.") {
        "manager"
    } else {
        "_manager"
    };

    format!(
        "use sea_orm_migration::prelude::*;\n\n#[derive(DeriveMigrationName)]\npub struct Migration;\n\n#[async_trait::async_trait]\nimpl MigrationTrait for Migration {{\n    async fn up(&self, {up_param}: &SchemaManager) -> Result<(), DbErr> {{\n{up}\n        Ok(())\n    }}\n\n    async fn down(&self, {down_param}: &SchemaManager) -> Result<(), DbErr> {{\n{down}\n        Ok(())\n    }}\n}}\n",
        up_param = up_param,
        down_param = down_param,
        up = up_body.trim_end(),
        down = down_body.trim_end()
    )
}

pub fn generate_batch_up_file(changes: &[&Changes], timestamp: &str) -> String {
    let mut body = String::new();
    for change in changes {
        append_up_ops(change, &mut body);
    }
    format!(
        "// Batch up - auto-generated by runique\n// Timestamp: {0}\n// Tables: {1}\nuse sea_orm_migration::prelude::*;\n\n#[derive(DeriveMigrationName)]\npub struct Migration;\n\n#[async_trait::async_trait]\nimpl MigrationTrait for Migration {{\n    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n{2}\n        Ok(())\n    }}\n\n    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {{\n        Ok(())\n    }}\n}}\n",
        timestamp,
        changes
            .iter()
            .map(|c| c.table_name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        body.trim_end()
    )
}

pub fn generate_batch_down_file(changes: &[&Changes], timestamp: &str) -> String {
    let mut body = String::new();
    for change in changes {
        append_down_ops(change, &mut body);
    }
    format!(
        "// Batch down - auto-generated by runique\n// Timestamp: {0}\n// Tables: {1}\nuse sea_orm_migration::prelude::*;\n\n#[derive(DeriveMigrationName)]\npub struct Migration;\n\n#[async_trait::async_trait]\nimpl MigrationTrait for Migration {{\n    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> {{\n        Ok(())\n    }}\n\n    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {{\n{2}\n        Ok(())\n    }}\n}}\n",
        timestamp,
        changes
            .iter()
            .map(|c| c.table_name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        body.trim_end()
    )
}

fn build_create_table_cols(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    let mut cols = String::new();

    if let Some(ref pk) = schema.primary_key {
        cols.push_str(&format!("{}\n", render_pk_col(pk)));
    }

    for col in schema.columns.iter().filter(|c| !c.ignored) {
        cols.push_str(&format!(
            "                    .col({})\n",
            render_column_def(col, db_kind)
        ));
    }

    cols
}

fn build_fk_create_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for fk in &schema.foreign_keys {
        out.push_str(&format!(
            "        manager\n            .create_foreign_key(\n                ForeignKey::create()\n                    .name(\"{table}_{from}_{to_table}_fkey\")\n                    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                    .on_delete(ForeignKeyAction::{on_delete})\n                    .on_update(ForeignKeyAction::{on_update})\n                    .to_owned(),\n            )\n            .await?;\n\n",
            table = schema.table_name,
            from = fk.from_column,
            to_table = fk.to_table,
            to_col = fk.to_column,
            on_delete = fk.on_delete,
            on_update = fk.on_update
        ));
    }
    out
}

fn build_index_create_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for idx in &schema.indexes {
        out.push_str(&render_create_index_stmt(
            &schema.table_name,
            &idx.name,
            &idx.columns,
            idx.unique,
        ));
        out.push('\n');
    }
    out
}

fn build_fk_drop_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for fk in &schema.foreign_keys {
        out.push_str(&format!(
            "        manager\n            .drop_foreign_key(\n                ForeignKey::drop()\n                    .table(Alias::new(\"{table}\"))\n                    .name(\"{table}_{from}_{to}_fkey\")\n                    .to_owned(),\n            )\n            .await?;\n\n",
            table = schema.table_name,
            from = fk.from_column,
            to = fk.to_table
        ));
    }
    out
}

fn build_index_drop_stmts(schema: &ParsedSchema) -> String {
    let mut out = String::new();
    for idx in &schema.indexes {
        out.push_str(&format!(
            "        manager\n            .drop_index(Index::drop().name(\"{idx}\").table(Alias::new(\"{table}\")).to_owned())\n            .await?;\n\n",
            idx = idx.name,
            table = schema.table_name
        ));
    }
    out
}

fn build_alter_bodies(change: &Changes) -> (String, String) {
    let mut up = String::new();
    let mut down = String::new();

    // 1) DROP indexes (up) / DROP added indexes (down)
    for idx in &change.dropped_indexes {
        push_drop_index(&mut up, &change.table_name, &idx.name);
    }
    for idx in &change.added_indexes {
        push_drop_index(&mut down, &change.table_name, &idx.name);
    }

    // 2) DROP FK
    for fk in &change.dropped_fks {
        push_drop_fk(&mut up, &change.table_name, &fk.from_column, &fk.to_table);
    }
    for fk in &change.added_fks {
        push_drop_fk(&mut down, &change.table_name, &fk.from_column, &fk.to_table);
    }

    // 3) DROP columns
    for col in &change.dropped_columns {
        push_drop_column(&mut up, &change.table_name, &col.name);
    }
    for col in &change.added_columns {
        push_drop_column(&mut down, &change.table_name, &col.name);
    }

    // 4) MODIFY columns
    for (old, new) in &change.modified_columns {
        // type change => manual
        if old.col_type != new.col_type {
            up.push_str(&format!(
                "        // WARNING: type change on column '{col}': {old} -> {new}\n        // Manual migration required.\n\n",
                col = new.name,
                old = old.col_type,
                new = new.col_type
            ));
            continue;
        }

        // nullable -> not_null => destructive unless you backfill
        if old.nullable && !new.nullable {
            // Generates modify_column anyway (risky if NULLs exist)
            push_modify_column(
                &mut up,
                &change.table_name,
                &new.name,
                &new.col_type,
                new.nullable,
                new.unique,
            );
            push_modify_column(
                &mut down,
                &change.table_name,
                &old.name,
                &old.col_type,
                old.nullable,
                old.unique,
            );
            continue;
        }

        // safe modify
        push_modify_column(
            &mut up,
            &change.table_name,
            &new.name,
            &new.col_type,
            new.nullable,
            new.unique,
        );
        push_modify_column(
            &mut down,
            &change.table_name,
            &old.name,
            &old.col_type,
            old.nullable,
            old.unique,
        );
    }

    // 5) ADD columns
    for col in &change.added_columns {
        push_add_column(&mut up, &change.table_name, col);
    }

    // 6) Recreate dropped columns in DOWN (now correct because we store ParsedColumn)
    for col in &change.dropped_columns {
        push_add_column(&mut down, &change.table_name, col);
    }

    // 7) ADD FK
    for fk in &change.added_fks {
        push_create_fk(
            &mut up,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }
    for fk in &change.dropped_fks {
        push_create_fk(
            &mut down,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }

    // 8) ADD indexes
    for idx in &change.added_indexes {
        push_create_index(
            &mut up,
            &change.table_name,
            &idx.name,
            &idx.columns,
            idx.unique,
        );
    }
    for idx in &change.dropped_indexes {
        push_create_index(
            &mut down,
            &change.table_name,
            &idx.name,
            &idx.columns,
            idx.unique,
        );
    }

    // 9) Enum renames (data migration)
    for (col, old_val, new_val) in &change.enum_renames {
        up.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"UPDATE {table} SET {col} = '{new}' WHERE {col} = '{old}'\"\n        ).await?;\n\n",
            table = change.table_name, col = col, old = old_val, new = new_val,
        ));
        // down: inverse
        down.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"UPDATE {table} SET {col} = '{old}' WHERE {col} = '{new}'\"\n        ).await?;\n\n",
            table = change.table_name, col = col, old = old_val, new = new_val,
        ));
    }

    // 10) Enum value additions/removals — manual migration required
    for (_col, enum_name, val) in &change.enum_value_adds {
        up.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"ALTER TYPE {enum_name} ADD VALUE IF NOT EXISTS '{val}'\"\n        ).await?;\n\n",
            enum_name = enum_name, val = val,
        ));
    }
    for (_col, enum_name, val) in &change.enum_value_drops {
        up.push_str(&format!(
            "        // WARNING: value '{val}' removed from {enum_name} — manual migration required.\n\n",
            val = val, enum_name = enum_name,
        ));
        down.push_str(&format!(
            "        manager.get_connection().execute_unprepared(\n            \"ALTER TYPE {enum_name} ADD VALUE IF NOT EXISTS '{val}'\"\n        ).await?;\n\n",
            enum_name = enum_name, val = val,
        ));
    }

    (up, down)
}

fn append_up_ops(change: &Changes, buf: &mut String) {
    for idx in &change.dropped_indexes {
        push_drop_index(buf, &change.table_name, &idx.name);
    }
    for fk in &change.dropped_fks {
        push_drop_fk(buf, &change.table_name, &fk.from_column, &fk.to_table);
    }
    for col in &change.dropped_columns {
        push_drop_column(buf, &change.table_name, &col.name);
    }
    for col in &change.added_columns {
        push_add_column(buf, &change.table_name, col);
    }
    for fk in &change.added_fks {
        push_create_fk(
            buf,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }
    for idx in &change.added_indexes {
        push_create_index(buf, &change.table_name, &idx.name, &idx.columns, idx.unique);
    }
}

fn append_down_ops(change: &Changes, buf: &mut String) {
    for idx in &change.added_indexes {
        push_drop_index(buf, &change.table_name, &idx.name);
    }
    for fk in &change.added_fks {
        push_drop_fk(buf, &change.table_name, &fk.from_column, &fk.to_table);
    }
    for col in &change.added_columns {
        push_drop_column(buf, &change.table_name, &col.name);
    }
    // recreate dropped columns
    for col in &change.dropped_columns {
        push_add_column(buf, &change.table_name, col);
    }
    for fk in &change.dropped_fks {
        push_create_fk(
            buf,
            &change.table_name,
            &fk.from_column,
            &fk.to_table,
            &fk.to_column,
            &fk.on_delete,
            &fk.on_update,
        );
    }
    for idx in &change.dropped_indexes {
        push_create_index(buf, &change.table_name, &idx.name, &idx.columns, idx.unique);
    }
}

fn render_pk_col(pk: &ParsedColumn) -> String {
    let ty = col_type_to_method(&pk.col_type);

    // auto_increment only makes sense for integer-ish PK.
    let autoinc_ok = matches!(
        pk.col_type.as_str(),
        "Integer" | "BigInteger" | "SmallInteger" | "TinyInteger" | "Unsigned" | "BigUnsigned"
    );

    let mut s = format!(
        "                    .col(ColumnDef::new(Alias::new(\"{name}\")).{ty}.not_null()",
        name = pk.name,
        ty = ty
    );

    if autoinc_ok {
        s.push_str(".auto_increment()");
    }

    s.push_str(".primary_key())");
    s
}

fn render_column_def(col: &ParsedColumn, db_kind: &DbKind) -> String {
    let null = if col.nullable {
        ".null()"
    } else {
        ".not_null()"
    };
    let uniq = if col.unique { ".unique_key()" } else { "" };
    let default = if col.has_default_now {
        ".default(Expr::current_timestamp())"
    } else {
        ""
    };
    let on_update = if col.updated_at && *db_kind == DbKind::Mysql {
        ".extra(\"ON UPDATE CURRENT_TIMESTAMP\")"
    } else {
        ""
    };

    if !col.enum_string_values.is_empty() {
        let name = col.enum_name.as_deref().unwrap_or(&col.name);
        let variants: Vec<String> = col
            .enum_string_values
            .iter()
            .map(|v| format!("Alias::new(\"{}\").into_iden()", v))
            .collect();

        format!(
            "ColumnDef::new_with_type(Alias::new(\"{name}\"), ColumnType::Enum {{ name: Alias::new(\"{enum_name}\").into_iden(), variants: vec![{variants}] }}){null}{uniq}",
            name = col.name,
            enum_name = name,
            variants = variants.join(", "),
            null = null,
            uniq = uniq,
        )
    } else {
        let ty = col_type_to_method(&col.col_type);
        format!(
            "ColumnDef::new(Alias::new(\"{name}\")).{ty}{null}{uniq}{default}{on_update}",
            name = col.name,
            ty = ty,
            null = null,
            uniq = uniq,
            default = default,
            on_update = on_update,
        )
    }
}

/// Generates PostgreSQL triggers for `updated_at` columns.
/// For MySQL, handling is inline via `.extra("ON UPDATE CURRENT_TIMESTAMP")`.
fn build_updated_at_trigger_stmts(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    if *db_kind != DbKind::Postgres {
        return String::new();
    }
    let updated_at_cols: Vec<_> = schema.columns.iter().filter(|c| c.updated_at).collect();
    if updated_at_cols.is_empty() {
        return String::new();
    }

    let table = &schema.table_name;
    let fn_name = format!("set_updated_at_{}", table);
    let trigger_name = format!("trg_{}_updated_at", table);

    format!(
        "        manager.get_connection().execute_unprepared(\n            \"CREATE OR REPLACE FUNCTION {fn_name}() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;\"\n        ).await?;\n        manager.get_connection().execute_unprepared(\n            \"CREATE TRIGGER {trigger_name} BEFORE UPDATE ON {table} FOR EACH ROW EXECUTE PROCEDURE {fn_name}();\"\n        ).await?;\n\n",
        fn_name = fn_name,
        trigger_name = trigger_name,
        table = table,
    )
}

/// Drops PostgreSQL `updated_at` triggers in the `down` block.
fn build_updated_at_trigger_drops(schema: &ParsedSchema, db_kind: &DbKind) -> String {
    if *db_kind != DbKind::Postgres {
        return String::new();
    }
    let has_updated_at = schema.columns.iter().any(|c| c.updated_at);
    if !has_updated_at {
        return String::new();
    }

    let table = &schema.table_name;
    let fn_name = format!("set_updated_at_{}", table);
    let trigger_name = format!("trg_{}_updated_at", table);

    format!(
        "        manager.get_connection().execute_unprepared(\n            \"DROP TRIGGER IF EXISTS {trigger_name} ON {table};\"\n        ).await?;\n        manager.get_connection().execute_unprepared(\n            \"DROP FUNCTION IF EXISTS {fn_name}();\"\n        ).await?;\n\n",
        trigger_name = trigger_name,
        table = table,
        fn_name = fn_name,
    )
}

fn push_drop_index(buf: &mut String, table: &str, idx_name: &str) {
    buf.push_str(&format!(
        "        manager\n            .drop_index(Index::drop().name(\"{idx}\").table(Alias::new(\"{table}\")).to_owned())\n            .await?;\n\n",
        idx = idx_name,
        table = table
    ));
}

fn push_drop_fk(buf: &mut String, table: &str, from_col: &str, to_table: &str) {
    buf.push_str(&format!(
        "        manager\n            .drop_foreign_key(\n                ForeignKey::drop()\n                    .table(Alias::new(\"{table}\"))\n                    .name(\"{table}_{from}_{to}_fkey\")\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        from = from_col,
        to = to_table
    ));
}

fn push_drop_column(buf: &mut String, table: &str, col: &str) {
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .drop_column(Alias::new(\"{col}\"))\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        col = col
    ));
}

fn push_modify_column(
    buf: &mut String,
    table: &str,
    col: &str,
    col_type: &str,
    nullable: bool,
    unique: bool,
) {
    let null = if nullable { ".null()" } else { ".not_null()" };
    let uniq = if unique { ".unique_key()" } else { "" };
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .modify_column(ColumnDef::new(Alias::new(\"{col}\")).{ty}{null}{uniq})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        col = col,
        ty = col_type_to_method(col_type),
        null = null,
        uniq = uniq
    ));
}

fn push_add_column(buf: &mut String, table: &str, col: &ParsedColumn) {
    buf.push_str(&format!(
        "        manager\n            .alter_table(\n                Table::alter()\n                    .table(Alias::new(\"{table}\"))\n                    .add_column({coldef})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        coldef = render_column_def(col, &DbKind::Other),
    ));
}

fn push_create_fk(
    buf: &mut String,
    table: &str,
    from_col: &str,
    to_table: &str,
    to_col: &str,
    on_delete: &str,
    on_update: &str,
) {
    buf.push_str(&format!(
        "        manager\n            .create_foreign_key(\n                ForeignKey::create()\n                    .name(\"{table}_{from}_{to_table}_fkey\")\n                    .from(Alias::new(\"{table}\"), Alias::new(\"{from}\"))\n                    .to(Alias::new(\"{to_table}\"), Alias::new(\"{to_col}\"))\n                    .on_delete(ForeignKeyAction::{on_delete})\n                    .on_update(ForeignKeyAction::{on_update})\n                    .to_owned(),\n            )\n            .await?;\n\n",
        table = table,
        from = from_col,
        to_table = to_table,
        to_col = to_col,
        on_delete = on_delete,
        on_update = on_update
    ));
}

fn push_create_index(
    buf: &mut String,
    table: &str,
    idx_name: &str,
    columns: &[String],
    unique: bool,
) {
    buf.push_str(&render_create_index_stmt(table, idx_name, columns, unique));
    buf.push('\n');
}

fn render_create_index_stmt(
    table: &str,
    idx_name: &str,
    columns: &[String],
    unique: bool,
) -> String {
    let mut cols_chain = String::new();
    for c in columns {
        cols_chain.push_str(&format!(
            "                    .col(Alias::new(\"{c}\"))\n",
            c = c
        ));
    }

    let uniq_line = if unique {
        "                    .unique_key()\n"
    } else {
        ""
    };

    format!(
        "        manager\n            .create_index(\n                Index::create()\n                    .name(\"{idx}\")\n                    .table(Alias::new(\"{table}\"))\n{cols}{uniq}                    .to_owned(),\n            )\n            .await?;\n",
        idx = idx_name,
        table = table,
        cols = cols_chain,
        uniq = uniq_line
    )
}
