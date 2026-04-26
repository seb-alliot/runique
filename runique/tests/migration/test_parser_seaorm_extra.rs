//! Tests supplémentaires — parser_seaorm (FK, indexes, types variés)

use runique::migration::utils::parser_seaorm::parse_seaorm_source;

const TABLE_WITH_FK: &str = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("posts"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_foreign_key(
                ForeignKey::create()
                    .from(Alias::new("posts"), Alias::new("user_id"))
                    .to(Alias::new("users"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Cascade)
                    .on_update(ForeignKeyAction::NoAction)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("posts")).to_owned()).await?;
        Ok(())
    }
}
"#;

const TABLE_WITH_INDEX: &str = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("articles"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("slug")).string().not_null().unique())
                    .index(
                        Index::create()
                            .name("idx_articles_slug")
                            .col(Alias::new("slug"))
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("articles")).to_owned()).await?;
        Ok(())
    }
}
"#;

const TABLE_WITH_MANY_TYPES: &str = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("products"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("name")).string().not_null())
                    .col(ColumnDef::new(Alias::new("price")).double().not_null())
                    .col(ColumnDef::new(Alias::new("active")).boolean().not_null())
                    .col(ColumnDef::new(Alias::new("description")).text().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("products")).to_owned()).await?;
        Ok(())
    }
}
"#;

const TABLE_WITH_UNIQUE_COL: &str = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("accounts"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null().unique())
                    .col(ColumnDef::new(Alias::new("username")).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("accounts")).to_owned()).await?;
        Ok(())
    }
}
"#;

// ═══════════════════════════════════════════════════════════════
// Table avec clé étrangère
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_avec_fk_nom() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    assert_eq!(schema.table_name, "posts");
}

#[test]
fn test_parse_table_avec_fk_colonnes() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    assert!(schema.columns.iter().any(|c| c.name == "title"));
    assert!(schema.columns.iter().any(|c| c.name == "user_id"));
}

#[test]
fn test_parse_fk_detectee() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    assert!(!schema.foreign_keys.is_empty(), "La FK doit être détectée");
}

#[test]
fn test_parse_fk_from_column() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.from_column, "user_id");
}

#[test]
fn test_parse_fk_to_table() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.to_table, "users");
}

#[test]
fn test_parse_fk_on_delete_cascade() {
    let schema = parse_seaorm_source(TABLE_WITH_FK).expect("doit réussir");
    let fk = &schema.foreign_keys[0];
    assert_eq!(fk.on_delete, "Cascade");
}

// ═══════════════════════════════════════════════════════════════
// Table avec index
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_avec_index_nom() {
    let schema = parse_seaorm_source(TABLE_WITH_INDEX).expect("doit réussir");
    assert_eq!(schema.table_name, "articles");
}

#[test]
fn test_parse_index_detecte() {
    let schema = parse_seaorm_source(TABLE_WITH_INDEX).expect("doit réussir");
    assert!(!schema.indexes.is_empty(), "L'index doit être détecté");
}

#[test]
fn test_parse_index_nom() {
    let schema = parse_seaorm_source(TABLE_WITH_INDEX).expect("doit réussir");
    let idx = &schema.indexes[0];
    assert_eq!(idx.name, "idx_articles_slug");
}

#[test]
fn test_parse_index_unique() {
    let schema = parse_seaorm_source(TABLE_WITH_INDEX).expect("doit réussir");
    let idx = &schema.indexes[0];
    assert!(idx.unique);
}

// ═══════════════════════════════════════════════════════════════
// Table avec types variés
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_types_varies_nom() {
    let schema = parse_seaorm_source(TABLE_WITH_MANY_TYPES).expect("doit réussir");
    assert_eq!(schema.table_name, "products");
}

#[test]
fn test_parse_table_types_varies_colonnes() {
    let schema = parse_seaorm_source(TABLE_WITH_MANY_TYPES).expect("doit réussir");
    let noms: Vec<_> = schema.columns.iter().map(|c| c.name.as_str()).collect();
    assert!(noms.contains(&"name"));
    assert!(noms.contains(&"price"));
    assert!(noms.contains(&"active"));
    assert!(noms.contains(&"description"));
}

#[test]
fn test_parse_table_types_varies_nullable() {
    let schema = parse_seaorm_source(TABLE_WITH_MANY_TYPES).expect("doit réussir");
    let desc = schema.columns.iter().find(|c| c.name == "description");
    assert!(desc.is_some());
    assert!(desc.unwrap().nullable);
}

#[test]
fn test_parse_table_types_varies_non_nullable() {
    let schema = parse_seaorm_source(TABLE_WITH_MANY_TYPES).expect("doit réussir");
    let name = schema.columns.iter().find(|c| c.name == "name");
    assert!(name.is_some());
    assert!(!name.unwrap().nullable);
}

// ═══════════════════════════════════════════════════════════════
// Table avec colonne unique
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_colonne_unique() {
    let schema = parse_seaorm_source(TABLE_WITH_UNIQUE_COL).expect("doit réussir");
    let email = schema.columns.iter().find(|c| c.name == "email");
    assert!(email.is_some());
    assert!(email.unwrap().unique, "email doit être unique");
}

#[test]
fn test_parse_colonne_non_unique() {
    let schema = parse_seaorm_source(TABLE_WITH_UNIQUE_COL).expect("doit réussir");
    let username = schema.columns.iter().find(|c| c.name == "username");
    assert!(username.is_some());
    assert!(!username.unwrap().unique);
}

#[test]
fn test_parse_pk_non_dans_colonnes() {
    let schema = parse_seaorm_source(TABLE_WITH_UNIQUE_COL).expect("doit réussir");
    assert!(!schema.columns.iter().any(|c| c.name == "id"));
    assert!(schema.primary_key.is_some());
}

// ═══════════════════════════════════════════════════════════════
// Snapshot avec enum_type (round-trip)
// ═══════════════════════════════════════════════════════════════

const TABLE_WITH_ENUM_TYPE: &str = r#"
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("articles"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("status")).enum_type("Status", vec!["Draft".to_string(), "Published".to_string(), "Archive".to_string()]).not_null())
                    .col(ColumnDef::new(Alias::new("title")).string().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Alias::new("articles")).to_owned()).await?;
        Ok(())
    }
}
"#;

#[test]
fn test_parse_enum_type_colonne_presente() {
    let schema = parse_seaorm_source(TABLE_WITH_ENUM_TYPE).expect("doit réussir");
    assert!(schema.columns.iter().any(|c| c.name == "status"));
}

#[test]
fn test_parse_enum_type_string_values_collectes() {
    let schema = parse_seaorm_source(TABLE_WITH_ENUM_TYPE).expect("doit réussir");
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.enum_string_values.len(), 3);
    assert!(status.enum_string_values.contains(&"Draft".to_string()));
    assert!(status.enum_string_values.contains(&"Published".to_string()));
    assert!(status.enum_string_values.contains(&"Archive".to_string()));
}

#[test]
fn test_parse_enum_type_enum_name_stocke() {
    let schema = parse_seaorm_source(TABLE_WITH_ENUM_TYPE).expect("doit réussir");
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.enum_name.as_deref(), Some("Status"));
}

#[test]
fn test_parse_enum_type_col_type_enum() {
    let schema = parse_seaorm_source(TABLE_WITH_ENUM_TYPE).expect("doit réussir");
    let status = schema.columns.iter().find(|c| c.name == "status").unwrap();
    assert_eq!(status.col_type, "Enum");
}

#[test]
fn test_parse_colonne_ordinaire_pas_de_enum_values() {
    let schema = parse_seaorm_source(TABLE_WITH_ENUM_TYPE).expect("doit réussir");
    let title = schema.columns.iter().find(|c| c.name == "title").unwrap();
    assert!(title.enum_string_values.is_empty());
    assert!(title.enum_name.is_none());
}
