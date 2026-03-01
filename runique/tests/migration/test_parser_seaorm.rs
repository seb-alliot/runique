//! Tests — migration/utils/parser_seaorm.rs
//! Couvre : parse_seaorm_source (table name, columns, FK, indexes)

use runique::migration::utils::parser_seaorm::parse_seaorm_source;

// ─── Fixtures SeaORM ─────────────────────────────────────────────────────────

const SIMPLE_TABLE: &str = r#"
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
                    .col(ColumnDef::new(Alias::new("id")).integer().primary_key())
                    .col(ColumnDef::new(Alias::new("username")).string().not_null())
                    .col(ColumnDef::new(Alias::new("email")).string().not_null())
                    .col(ColumnDef::new(Alias::new("bio")).text().null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("users")).to_owned())
            .await?;
        Ok(())
    }
}
"#;

const INVALID_RUST: &str = "this is not valid rust code !!!@@@";

const NO_TABLE_NAME: &str = r#"
use sea_orm_migration::prelude::*;
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, _manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
}
"#;

// ═══════════════════════════════════════════════════════════════
// Source invalide
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_source_invalide_retourne_err() {
    let result = parse_seaorm_source(INVALID_RUST);
    assert!(result.is_err(), "Le code invalide doit retourner Err");
}

#[test]
fn test_parse_source_vide_retourne_err() {
    let result = parse_seaorm_source("");
    assert!(result.is_err(), "Une source vide doit retourner Err");
}

#[test]
fn test_parse_sans_table_name_retourne_err() {
    let result = parse_seaorm_source(NO_TABLE_NAME);
    assert!(
        result.is_err(),
        "Sans nom de table, le parse doit retourner Err"
    );
}

// ═══════════════════════════════════════════════════════════════
// Table simple
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_parse_nom_table() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    assert_eq!(schema.table_name, "users");
}

#[test]
fn test_parse_cle_primaire() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    assert!(schema.primary_key.is_some(), "La PK doit être détectée");
    assert_eq!(schema.primary_key.unwrap().name, "id");
}

#[test]
fn test_parse_colonnes_not_null() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    let username = schema.columns.iter().find(|c| c.name == "username");
    assert!(
        username.is_some(),
        "La colonne 'username' doit être présente"
    );
    assert!(!username.unwrap().nullable);
}

#[test]
fn test_parse_colonne_nullable() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    let bio = schema.columns.iter().find(|c| c.name == "bio");
    assert!(bio.is_some(), "La colonne 'bio' doit être présente");
    assert!(bio.unwrap().nullable, "La colonne 'bio' doit être nullable");
}

#[test]
fn test_parse_colonnes_non_vide() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    assert!(
        !schema.columns.is_empty(),
        "Il doit y avoir au moins des colonnes"
    );
}

#[test]
fn test_parse_pas_de_fk_dans_table_simple() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    assert!(schema.foreign_keys.is_empty(), "Pas de FK attendue");
}

#[test]
fn test_parse_pas_d_index_dans_table_simple() {
    let schema = parse_seaorm_source(SIMPLE_TABLE).expect("doit réussir");
    assert!(schema.indexes.is_empty(), "Pas d'index attendu");
}
