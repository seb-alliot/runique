use runique::migration::*;

// ============================================================
// Helpers
// ============================================================

fn make_schema(table: &str, pk: Option<&str>, cols: Vec<(&str, &str, bool, bool)>) -> ParsedSchema {
    ParsedSchema {
        table_name: table.to_string(),
        primary_key: pk.map(|n| ParsedColumn {
            name: n.to_string(),
            col_type: "Integer".to_string(),
            nullable: false,
            unique: false,
            ignored: false,
        }),
        columns: cols
            .iter()
            .map(|(name, col_type, nullable, unique)| ParsedColumn {
                name: name.to_string(),
                col_type: col_type.to_string(),
                nullable: *nullable,
                unique: *unique,
                ignored: false,
            })
            .collect(),
        foreign_keys: vec![],
        indexes: vec![],
    }
}

fn make_fk(from: &str, to_table: &str, to_col: &str) -> ParsedFk {
    ParsedFk {
        from_column: from.to_string(),
        to_table: to_table.to_string(),
        to_column: to_col.to_string(),
        on_delete: "Cascade".to_string(),
        on_update: "NoAction".to_string(),
    }
}

fn make_index(name: &str, cols: Vec<&str>, unique: bool) -> ParsedIndex {
    ParsedIndex {
        name: name.to_string(),
        columns: cols.iter().map(|s| s.to_string()).collect(),
        unique,
    }
}

fn diff(previous: &ParsedSchema, current: &ParsedSchema) -> Changes {
    let pk_name = current.primary_key.as_ref().map(|pk| pk.name.as_str());

    let prev_cols: std::collections::HashSet<&str> = previous
        .columns
        .iter()
        .filter(|c| Some(c.name.as_str()) != pk_name)
        .map(|c| c.name.as_str())
        .collect();

    let curr_cols: std::collections::HashSet<&str> = current
        .columns
        .iter()
        .filter(|c| Some(c.name.as_str()) != pk_name)
        .map(|c| c.name.as_str())
        .collect();

    let added_columns = current
        .columns
        .iter()
        .filter(|c| !prev_cols.contains(c.name.as_str()) && Some(c.name.as_str()) != pk_name)
        .cloned()
        .collect();

    let dropped_columns: Vec<ParsedColumn> = previous
        .columns
        .iter()
        .filter(|c| !c.ignored)
        .filter(|c| !curr_cols.contains(c.name.as_str()))
        .cloned()
        .collect();

    Changes {
        table_name: current.table_name.clone(),
        added_columns,
        dropped_columns,
        modified_columns: vec![],
        is_new_table: false,
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
    }
}

// ============================================================
// Parser builder
// ============================================================

#[test]
fn test_parse_builder_simple() {
    let source = r#"
        pub fn users_schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("username").string().required())
                .column(ColumnDef::new("email").string().required())
                .column(ColumnDef::new("password").string().required())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source);
    assert!(schema.is_some());
    let schema = schema.unwrap();
    assert_eq!(schema.table_name, "users");
    assert!(schema.primary_key.is_some());
    assert_eq!(schema.primary_key.unwrap().name, "id");
    assert_eq!(schema.columns.len(), 3);
    assert_eq!(schema.columns[0].name, "username");
    assert!(!schema.columns[0].nullable);
}

#[test]
fn test_parse_builder_nullable() {
    let source = r#"
        pub fn users_schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("created_at").datetime().nullable())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.columns[0].name, "created_at");
    assert!(schema.columns[0].nullable);
    assert_eq!(schema.columns[0].col_type, "DateTime");
}

#[test]
fn test_parse_builder_unique() {
    let source = r#"
        pub fn users_schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("email").string().required().unique())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert!(schema.columns[0].unique);
}

#[test]
fn test_parse_builder_model_macro_snake_case() {
    let source = r#"
        pub fn schema() -> ModelSchema {
            model!("EihwazUsers")
                .table_name("eihwaz_users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("username").string().required())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.table_name, "eihwaz_users");
}

#[test]
fn test_parse_builder_no_build_returns_none() {
    // Without .build(), the string should not be recognized
    let source = r#"
        pub fn schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("username").string().required())
        }
    "#;

    assert!(parse_schema_from_source(source).is_none());
}

#[test]
fn test_parse_builder_auto_now_is_datetime_nullable() {
    let source = r#"
        pub fn schema() -> ModelSchema {
            model!("posts")
                .table_name("posts")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("created_at").auto_now())
                .column(ColumnDef::new("updated_at").auto_now_update())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    let created = schema
        .columns
        .iter()
        .find(|c| c.name == "created_at")
        .unwrap();
    let updated = schema
        .columns
        .iter()
        .find(|c| c.name == "updated_at")
        .unwrap();
    assert_eq!(created.col_type, "DateTime");
    assert!(created.nullable, "auto_now should be nullable");
    assert_eq!(updated.col_type, "DateTime");
    assert!(updated.nullable, "auto_now_update should be nullable");
}

#[test]
fn test_parse_builder_pk_uuid() {
    let source = r#"
        pub fn schema() -> ModelSchema {
            model!("tokens")
                .table_name("tokens")
                .primary_key(PrimaryKeyDef::new("id").uuid())
                .column(ColumnDef::new("value").string().required())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    let pk = schema.primary_key.unwrap();
    assert_eq!(pk.col_type, "Uuid");
}

#[test]
fn test_parse_builder_pk_i64_is_biginteger() {
    let source = r#"
        pub fn schema() -> ModelSchema {
            model!("events")
                .table_name("events")
                .primary_key(PrimaryKeyDef::new("id").i64().auto_increment())
                .column(ColumnDef::new("name").string().required())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    let pk = schema.primary_key.unwrap();
    assert_eq!(pk.col_type, "BigInteger");
}

#[test]
fn test_parse_builder_ignored_column() {
    let source = r#"
        pub fn users_schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("username").string().required())
                .column(ColumnDef::new("full_name").string().ignored())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.columns.len(), 2);
    let ignored = schema
        .columns
        .iter()
        .find(|c| c.name == "full_name")
        .unwrap();
    assert!(ignored.ignored);
}

#[test]
fn test_parse_builder_foreign_key() {
    let source = r#"
        pub fn posts_schema() -> ModelSchema {
            model!("posts")
                .table_name("posts")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("title").string().required())
                .column(ColumnDef::new("user_id").integer().required())
                .foreign_key(
                    ForeignKeyDef::new("user_id")
                        .references("users", "id")
                        .on_delete(ForeignKeyAction::Cascade)
                )
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.foreign_keys.len(), 1);
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.from_column, "user_id");
    assert_eq!(fk.to_table, "users");
    assert_eq!(fk.to_column, "id");
    assert_eq!(fk.on_delete, "Cascade");
}

#[test]
fn test_parse_builder_foreign_key_no_action() {
    let source = r#"
        pub fn posts_schema() -> ModelSchema {
            model!("posts")
                .table_name("posts")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("category_id").integer().required())
                .foreign_key(
                    ForeignKeyDef::new("category_id")
                        .references("categories", "id")
                )
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.foreign_keys[0].on_delete, "NoAction");
}

#[test]
fn test_parse_builder_index_simple() {
    let source = r#"
        pub fn users_schema() -> ModelSchema {
            model!("users")
                .table_name("users")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("email").string().required())
                .index(IndexDef::new("idx_users_email").column("email").unique())
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert_eq!(schema.indexes.len(), 1);
    let idx = &schema.indexes[0];
    assert_eq!(idx.name, "idx_users_email");
    assert_eq!(idx.columns, vec!["email"]);
    assert!(idx.unique);
}

#[test]
fn test_parse_builder_index_composite() {
    let source = r#"
        pub fn posts_schema() -> ModelSchema {
            model!("posts")
                .table_name("posts")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("user_id").integer().required())
                .column(ColumnDef::new("slug").string().required())
                .index(IndexDef::new("idx_posts_user_slug").column("user_id").column("slug"))
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    let idx = &schema.indexes[0];
    assert_eq!(idx.name, "idx_posts_user_slug");
    assert_eq!(idx.columns.len(), 2);
    assert!(idx.columns.contains(&"user_id".to_string()));
    assert!(idx.columns.contains(&"slug".to_string()));
    assert!(!idx.unique);
}

#[test]
fn test_parse_builder_index_not_unique_by_default() {
    let source = r#"
        pub fn posts_schema() -> ModelSchema {
            model!("posts")
                .table_name("posts")
                .primary_key(PrimaryKeyDef::new("id").i32().auto_increment())
                .column(ColumnDef::new("status").string().required())
                .index(IndexDef::new("idx_posts_status").column("status"))
                .build()
                .unwrap()
        }
    "#;

    let schema = parse_schema_from_source(source).unwrap();
    assert!(!schema.indexes[0].unique);
}

// ============================================================
// SeaORM Parser
// ============================================================

#[test]
fn test_parse_seaorm_create_file() {
    let source = r#"
        use sea_orm_migration::prelude::*;
        #[derive(DeriveMigrationName)]
        pub struct Migration;
        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Table::create()
                            .table(Alias::new("users"))
                            .if_not_exists()
                            .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                            .col(ColumnDef::new(Alias::new("username")).string().not_null())
                            .col(ColumnDef::new(Alias::new("email")).string().not_null().unique())
                            .col(ColumnDef::new(Alias::new("created_at")).date_time().null())
                            .to_owned(),
                    )
                    .await?;
                Ok(())
            }
            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager.drop_table(Table::drop().table(Alias::new("users")).to_owned()).await?;
                Ok(())
            }
        }
    "#;

    let schema = parse_seaorm_source(source).unwrap();
    assert_eq!(schema.table_name, "users");
    assert!(schema.primary_key.is_some());
    assert_eq!(schema.primary_key.unwrap().name, "id");
    assert_eq!(schema.columns.len(), 3);
    let email = schema.columns.iter().find(|c| c.name == "email").unwrap();
    assert!(email.unique);
    let created_at = schema
        .columns
        .iter()
        .find(|c| c.name == "created_at")
        .unwrap();
    assert!(created_at.nullable);
}

#[test]
fn test_parse_seaorm_pk_not_in_columns() {
    let source = r#"
        use sea_orm_migration::prelude::*;
        #[derive(DeriveMigrationName)]
        pub struct Migration;
        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager
                    .create_table(
                        Table::create()
                            .table(Alias::new("users"))
                            .if_not_exists()
                            .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                            .col(ColumnDef::new(Alias::new("username")).string().not_null())
                            .to_owned(),
                    )
                    .await?;
                Ok(())
            }
            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
        }
    "#;

    let schema = parse_seaorm_source(source).unwrap();
    assert!(
        !schema.columns.iter().any(|c| c.name == "id"),
        "PK should not appear in columns"
    );
}

#[test]
fn test_parse_seaorm_foreign_key() {
    let source = r#"
        use sea_orm_migration::prelude::*;
        #[derive(DeriveMigrationName)]
        pub struct Migration;
        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager.create_table(Table::create().table(Alias::new("posts")).if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .to_owned()).await?;
                manager.create_foreign_key(
                    ForeignKey::create()
                        .from(Alias::new("posts"), Alias::new("user_id"))
                        .to(Alias::new("users"), Alias::new("id"))
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::NoAction)
                        .to_owned(),
                ).await?;
                Ok(())
            }
            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
        }
    "#;

    let schema = parse_seaorm_source(source).unwrap();
    assert_eq!(schema.foreign_keys.len(), 1);
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.from_column, "user_id");
    assert_eq!(fk.to_table, "users");
    assert_eq!(fk.to_column, "id");
    assert_eq!(fk.on_delete, "Cascade");
}

#[test]
fn test_parse_seaorm_index() {
    let source = r#"
        use sea_orm_migration::prelude::*;
        #[derive(DeriveMigrationName)]
        pub struct Migration;
        #[async_trait::async_trait]
        impl MigrationTrait for Migration {
            async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
                manager.create_table(Table::create().table(Alias::new("users")).if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null())
                    .to_owned()).await?;
                manager.create_index(
                    Index::create()
                        .name("idx_users_email")
                        .table(Alias::new("users"))
                        .col(Alias::new("email"))
                        .unique()
                        .to_owned(),
                ).await?;
                Ok(())
            }
            async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
        }
    "#;

    let schema = parse_seaorm_source(source).unwrap();
    assert_eq!(schema.indexes.len(), 1);
    let idx = &schema.indexes[0];
    assert_eq!(idx.name, "idx_users_email");
    assert!(idx.unique);
}

// ============================================================
// Diff — columns (via local helper)
// ============================================================

#[test]
fn test_diff_add_column() {
    let previous = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );
    let current = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
            ("is_active", "Boolean", false, false),
        ],
    );

    let changes = diff(&previous, &current);
    assert!(!changes.is_new_table);
    assert_eq!(changes.added_columns.len(), 1);
    assert_eq!(changes.added_columns[0].name, "is_active");
    assert!(changes.dropped_columns.is_empty());
}

#[test]
fn test_diff_drop_column() {
    let previous = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
            ("roles", "Text", true, false),
        ],
    );
    let current = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );

    let changes = diff(&previous, &current);
    assert_eq!(changes.dropped_columns.len(), 1);
    assert_eq!(changes.dropped_columns[0].name, "roles");
    assert!(changes.added_columns.is_empty());
}

#[test]
fn test_diff_pk_never_in_changes() {
    let previous = make_schema(
        "users",
        Some("id"),
        vec![("username", "String", false, false)],
    );
    let current = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );

    let changes = diff(&previous, &current);
    assert!(!changes.added_columns.iter().any(|c| c.name == "id"));
    assert!(!changes.dropped_columns.iter().any(|c| c.name == "id"));
}

#[test]
fn test_diff_no_changes() {
    let schema = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );
    let changes = diff(&schema, &schema);
    assert!(changes.is_empty());
}

#[test]
fn test_diff_unique_preserved() {
    let previous = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    let current = make_schema(
        "users",
        Some("id"),
        vec![
            ("email", "String", false, false),
            ("username", "String", false, true),
        ],
    );

    let changes = diff(&previous, &current);
    let username = changes
        .added_columns
        .iter()
        .find(|c| c.name == "username")
        .unwrap();
    assert!(username.unique);
}

// ============================================================
// Diff — columns via real diff_schemas (modified_columns)
// ============================================================

#[test]
fn test_diff_schemas_modified_nullable() {
    let previous = make_schema("users", Some("id"), vec![("bio", "Text", true, false)]);
    let current = make_schema("users", Some("id"), vec![("bio", "Text", false, false)]);

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.modified_columns.len(), 1);
    let (old, new) = &changes.modified_columns[0];
    assert!(old.nullable);
    assert!(!new.nullable);
    assert_eq!(old.col_type, new.col_type);
}

#[test]
fn test_diff_schemas_modified_type() {
    let previous = make_schema("users", Some("id"), vec![("age", "Integer", false, false)]);
    let current = make_schema(
        "users",
        Some("id"),
        vec![("age", "BigInteger", false, false)],
    );

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.modified_columns.len(), 1);
    let (old, new) = &changes.modified_columns[0];
    assert_eq!(old.col_type, "Integer");
    assert_eq!(new.col_type, "BigInteger");
}

#[test]
fn test_diff_schemas_modified_unique() {
    let previous = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    let current = make_schema("users", Some("id"), vec![("email", "String", false, true)]);

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.modified_columns.len(), 1);
    let (old, new) = &changes.modified_columns[0];
    assert!(!old.unique);
    assert!(new.unique);
}

#[test]
fn test_diff_schemas_no_false_positives() {
    // Same schema: no modified_columns
    let schema = make_schema(
        "users",
        Some("id"),
        vec![
            ("email", "String", false, true),
            ("bio", "Text", true, false),
        ],
    );
    let changes = diff_schemas(&schema, &schema);
    assert!(changes.modified_columns.is_empty());
    assert!(changes.added_columns.is_empty());
    assert!(changes.dropped_columns.is_empty());
}

// ============================================================
// Diff — FK via real diff_schemas
// ============================================================

#[test]
fn test_diff_schemas_fk_added() {
    let previous = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );
    let mut current = previous.clone();
    current.foreign_keys.push(make_fk("user_id", "users", "id"));

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.added_fks.len(), 1);
    assert!(changes.dropped_fks.is_empty());
    let fk = &changes.added_fks[0];
    assert_eq!(fk.from_column, "user_id");
    assert_eq!(fk.to_table, "users");
}

#[test]
fn test_diff_schemas_fk_dropped() {
    let mut previous = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );
    previous
        .foreign_keys
        .push(make_fk("user_id", "users", "id"));
    let current = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.dropped_fks.len(), 1);
    assert!(changes.added_fks.is_empty());
}

#[test]
fn test_diff_schemas_fk_unchanged() {
    let mut schema = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );
    schema.foreign_keys.push(make_fk("user_id", "users", "id"));

    let changes = diff_schemas(&schema, &schema);
    assert!(changes.added_fks.is_empty());
    assert!(changes.dropped_fks.is_empty());
}

#[test]
fn test_diff_schemas_fk_replaced() {
    // Remplace une FK par une autre
    let mut previous = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );
    previous
        .foreign_keys
        .push(make_fk("user_id", "users", "id"));
    let mut current = previous.clone();
    current.foreign_keys.clear();
    current
        .foreign_keys
        .push(make_fk("user_id", "admins", "id"));

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.added_fks.len(), 1);
    assert_eq!(changes.dropped_fks.len(), 1);
    assert_eq!(changes.added_fks[0].to_table, "admins");
    assert_eq!(changes.dropped_fks[0].to_table, "users");
}

// ============================================================
// Diff — indexes via real diff_schemas
// ============================================================

#[test]
fn test_diff_schemas_index_added() {
    let previous = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    let mut current = previous.clone();
    current
        .indexes
        .push(make_index("idx_users_email", vec!["email"], true));

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.added_indexes.len(), 1);
    assert!(changes.dropped_indexes.is_empty());
    assert_eq!(changes.added_indexes[0].name, "idx_users_email");
}

#[test]
fn test_diff_schemas_index_dropped() {
    let mut previous = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    previous
        .indexes
        .push(make_index("idx_users_email", vec!["email"], true));
    let current = make_schema("users", Some("id"), vec![("email", "String", false, false)]);

    let changes = diff_schemas(&previous, &current);
    assert_eq!(changes.dropped_indexes.len(), 1);
    assert!(changes.added_indexes.is_empty());
}

#[test]
fn test_diff_schemas_index_unchanged() {
    let mut schema = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    schema
        .indexes
        .push(make_index("idx_users_email", vec!["email"], true));

    let changes = diff_schemas(&schema, &schema);
    assert!(changes.added_indexes.is_empty());
    assert!(changes.dropped_indexes.is_empty());
}

// ============================================================
// Changes::is_empty()
// ============================================================

#[test]
fn test_changes_is_empty_true_when_all_empty() {
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    };
    assert!(changes.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_new_table() {
    let changes = Changes {
        table_name: "users".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: true,
    };
    assert!(!changes.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_added_column() {
    let mut c = empty_changes();
    c.added_columns.push(make_col("x"));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_dropped_column() {
    let mut c = empty_changes();
    c.dropped_columns.push(make_col("x"));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_modified_column() {
    let mut c = empty_changes();
    c.modified_columns.push((make_col("x"), make_col("x")));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_added_fk() {
    let mut c = empty_changes();
    c.added_fks.push(make_fk("a", "b", "id"));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_dropped_fk() {
    let mut c = empty_changes();
    c.dropped_fks.push(make_fk("a", "b", "id"));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_added_index() {
    let mut c = empty_changes();
    c.added_indexes.push(make_index("idx", vec!["col"], false));
    assert!(!c.is_empty());
}

#[test]
fn test_changes_is_empty_false_if_dropped_index() {
    let mut c = empty_changes();
    c.dropped_indexes
        .push(make_index("idx", vec!["col"], false));
    assert!(!c.is_empty());
}

fn empty_changes() -> Changes {
    Changes {
        table_name: "t".to_string(),
        added_columns: vec![],
        dropped_columns: vec![],
        modified_columns: vec![],
        added_fks: vec![],
        dropped_fks: vec![],
        added_indexes: vec![],
        dropped_indexes: vec![],
        is_new_table: false,
    }
}

fn make_col(name: &str) -> ParsedColumn {
    ParsedColumn {
        name: name.to_string(),
        col_type: "String".to_string(),
        nullable: false,
        unique: false,
        ignored: false,
    }
}

// ============================================================
// db_columns
// ============================================================

#[test]
fn test_db_columns_excludes_pk() {
    let schema = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );
    let cols = db_columns(&schema);
    assert!(!cols.iter().any(|c| c.name == "id"));
    assert_eq!(cols.len(), 2);
}

#[test]
fn test_db_columns_excludes_ignored() {
    let mut schema = make_schema(
        "users",
        Some("id"),
        vec![("username", "String", false, false)],
    );
    schema.columns.push(ParsedColumn {
        name: "computed".to_string(),
        col_type: "String".to_string(),
        nullable: false,
        unique: false,
        ignored: true,
    });
    let cols = db_columns(&schema);
    assert!(!cols.iter().any(|c| c.name == "computed"));
    assert_eq!(cols.len(), 1);
}

#[test]
fn test_db_columns_no_pk_no_filter() {
    let schema = make_schema(
        "users",
        None,
        vec![
            ("username", "String", false, false),
            ("email", "String", false, false),
        ],
    );
    let cols = db_columns(&schema);
    assert_eq!(cols.len(), 2);
}

// ============================================================
// Helpers — to_snake_case
// ============================================================

#[test]
fn test_to_snake_case_already_snake() {
    assert_eq!(to_snake_case("eihwaz_users"), "eihwaz_users");
}

#[test]
fn test_to_snake_case_pascal() {
    assert_eq!(to_snake_case("EihwazUsers"), "eihwaz_users");
}

#[test]
fn test_to_snake_case_single_word() {
    assert_eq!(to_snake_case("Users"), "users");
}

#[test]
fn test_to_snake_case_all_caps_each_word() {
    assert_eq!(to_snake_case("MyBlogPost"), "my_blog_post");
}

#[test]
fn test_to_snake_case_lowercase_unchanged() {
    assert_eq!(to_snake_case("posts"), "posts");
}

// ============================================================
// Helpers — col_type_to_method
// ============================================================

#[test]
fn test_col_type_to_method_all_types() {
    assert_eq!(col_type_to_method("Text"), "text()");
    assert_eq!(col_type_to_method("Integer"), "integer()");
    assert_eq!(col_type_to_method("BigInteger"), "big_integer()");
    assert_eq!(col_type_to_method("Boolean"), "boolean()");
    assert_eq!(col_type_to_method("DateTime"), "date_time()");
    assert_eq!(col_type_to_method("Uuid"), "uuid()");
    assert_eq!(col_type_to_method("Json"), "json()");
}

#[test]
fn test_col_type_to_method_fallback_string() {
    assert_eq!(col_type_to_method("String"), "string()");
    assert_eq!(col_type_to_method("unknown"), "string()");
    assert_eq!(col_type_to_method(""), "string()");
}

// ============================================================
// Generators — generate_create_file
// ============================================================

#[test]
fn test_generate_create_file_contains_table_name() {
    let schema = make_schema("posts", Some("id"), vec![("title", "String", false, false)]);
    let output = generate_create_file(&schema);
    assert!(output.contains("Alias::new(\"posts\")"));
    assert!(output.contains("create_table"));
    assert!(output.contains("drop_table"));
}

#[test]
fn test_generate_create_file_contains_columns() {
    let schema = make_schema(
        "users",
        Some("id"),
        vec![
            ("username", "String", false, false),
            ("bio", "Text", true, false),
            ("email", "String", false, true),
        ],
    );
    let output = generate_create_file(&schema);
    assert!(output.contains("Alias::new(\"username\")"));
    assert!(output.contains("Alias::new(\"bio\")"));
    assert!(
        output.contains(".null()"),
        "nullable column should use .null()"
    );
    assert!(
        output.contains(".unique()"),
        "unique column should use .unique()"
    );
    assert!(output.contains("Alias::new(\"id\")"), "PK should appear");
    assert!(output.contains("primary_key()"));
    assert!(output.contains("auto_increment()"));
}

#[test]
fn test_generate_create_file_ignored_column_excluded() {
    let mut schema = make_schema(
        "users",
        Some("id"),
        vec![("username", "String", false, false)],
    );
    schema.columns.push(ParsedColumn {
        name: "computed".to_string(),
        col_type: "String".to_string(),
        nullable: false,
        unique: false,
        ignored: true,
    });
    let output = generate_create_file(&schema);
    assert!(!output.contains("Alias::new(\"computed\")"));
}

#[test]
fn test_generate_create_file_with_fk() {
    let mut schema = make_schema(
        "posts",
        Some("id"),
        vec![("user_id", "Integer", false, false)],
    );
    schema.foreign_keys.push(make_fk("user_id", "users", "id"));
    let output = generate_create_file(&schema);
    assert!(output.contains("create_foreign_key"));
    assert!(output.contains("ForeignKeyAction::Cascade"));
    assert!(output.contains("drop_foreign_key"));
}

#[test]
fn test_generate_create_file_with_index() {
    let mut schema = make_schema("users", Some("id"), vec![("email", "String", false, false)]);
    schema
        .indexes
        .push(make_index("idx_users_email", vec!["email"], true));
    let output = generate_create_file(&schema);
    assert!(output.contains("create_index"));
    assert!(output.contains("idx_users_email"));
    assert!(output.contains(".unique()"));
    assert!(output.contains("drop_index"));
}

// ============================================================
// Generators — generate_alter_file
// ============================================================

#[test]
fn test_generate_alter_file_add_column() {
    let mut c = empty_changes();
    c.added_columns.push(ParsedColumn {
        name: "is_active".to_string(),
        col_type: "Boolean".to_string(),
        nullable: false,
        unique: false,
        ignored: false,
    });
    let output = generate_alter_file(&c);
    // up() contains add_column
    let up_section = extract_up(&output);
    assert!(up_section.contains("add_column"));
    assert!(up_section.contains("Alias::new(\"is_active\")"));
    assert!(up_section.contains("boolean()"));
    // down() contains drop_column
    let down_section = extract_down(&output);
    assert!(down_section.contains("drop_column"));
    assert!(down_section.contains("Alias::new(\"is_active\")"));
}

#[test]
fn test_generate_alter_file_drop_column() {
    let mut c = empty_changes();
    c.dropped_columns.push(make_col("old_field"));
    let output = generate_alter_file(&c);
    let up_section = extract_up(&output);
    assert!(up_section.contains("drop_column"));
    assert!(up_section.contains("Alias::new(\"old_field\")"));
    // down() recreates the column
    let down_section = extract_down(&output);
    assert!(down_section.contains("add_column"));
    assert!(down_section.contains("Alias::new(\"old_field\")"));
}

#[test]
fn test_generate_alter_file_add_fk() {
    let mut c = empty_changes();
    c.added_fks.push(make_fk("user_id", "users", "id"));
    let output = generate_alter_file(&c);
    let up_section = extract_up(&output);
    assert!(up_section.contains("create_foreign_key"));
    let down_section = extract_down(&output);
    assert!(down_section.contains("drop_foreign_key"));
}

#[test]
fn test_generate_alter_file_drop_fk() {
    let mut c = empty_changes();
    c.dropped_fks.push(make_fk("user_id", "users", "id"));
    let output = generate_alter_file(&c);
    let up_section = extract_up(&output);
    assert!(up_section.contains("drop_foreign_key"));
    let down_section = extract_down(&output);
    assert!(down_section.contains("create_foreign_key"));
}

#[test]
fn test_generate_alter_file_add_index() {
    let mut c = empty_changes();
    c.added_indexes
        .push(make_index("idx_test", vec!["col_a"], false));
    let output = generate_alter_file(&c);
    let up_section = extract_up(&output);
    assert!(up_section.contains("create_index"));
    assert!(up_section.contains("idx_test"));
    let down_section = extract_down(&output);
    assert!(down_section.contains("drop_index"));
}

#[test]
fn test_generate_alter_file_type_change_is_comment() {
    let mut c = empty_changes();
    c.modified_columns.push((
        ParsedColumn {
            name: "age".to_string(),
            col_type: "Integer".to_string(),
            nullable: false,
            unique: false,
            ignored: false,
        },
        ParsedColumn {
            name: "age".to_string(),
            col_type: "BigInteger".to_string(),
            nullable: false,
            unique: false,
            ignored: false,
        },
    ));
    let output = generate_alter_file(&c);
    assert!(
        output.contains("WARNING"),
        "type change should generate a WARNING comment"
    );
    assert!(output.contains("Manual migration required"));
}

#[test]
fn test_generate_alter_file_nullable_change_uses_modify() {
    let mut c = empty_changes();
    c.modified_columns.push((
        ParsedColumn {
            name: "bio".to_string(),
            col_type: "Text".to_string(),
            nullable: true,
            unique: false,
            ignored: false,
        },
        ParsedColumn {
            name: "bio".to_string(),
            col_type: "Text".to_string(),
            nullable: false,
            unique: false,
            ignored: false,
        },
    ));
    let output = generate_alter_file(&c);
    let up_section = extract_up(&output);
    assert!(up_section.contains("modify_column"));
    assert!(up_section.contains(".not_null()"));
}

#[test]
fn test_generate_alter_file_valid_rust_structure() {
    // Checks that the generated file contains the expected structural elements
    let c = empty_changes();
    let output = generate_alter_file(&c);
    assert!(output.contains("use sea_orm_migration::prelude::*;"));
    assert!(output.contains("impl MigrationTrait for Migration"));
    assert!(output.contains("async fn up"));
    assert!(output.contains("async fn down"));
    assert!(output.contains("Ok(())"));
}

// ── helpers to extract up/down ──────────────────────────────────────────────

fn extract_up(src: &str) -> &str {
    let start = src.find("async fn up").unwrap_or(0);
    let end = src.find("async fn down").unwrap_or(src.len());
    &src[start..end]
}

fn extract_down(src: &str) -> &str {
    let start = src.find("async fn down").unwrap_or(0);
    &src[start..]
}

// ============================================================
// Diff — ignored via real diff_schemas
// ============================================================

#[test]
fn test_diff_schemas_ignored_column_not_in_changes() {
    let previous = make_schema("users", None, vec![("username", "String", false, false)]);
    let mut current = previous.clone();
    current.columns.push(ParsedColumn {
        name: "computed".to_string(),
        col_type: "String".to_string(),
        nullable: true,
        unique: false,
        ignored: true,
    });

    let changes = diff_schemas(&previous, &current);
    assert!(
        changes.added_columns.is_empty(),
        "ignored column must not appear in added_columns"
    );
}
