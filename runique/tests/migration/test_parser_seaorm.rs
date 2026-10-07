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

// ═══════════════════════════════════════════════════════════════
// Snapshot complet — écrit depuis les survivants cargo-mutants (2026-10-07)
// Même forme que ce que `makemigrations` génère (cf. demo-app/migration/src/snapshots).
// ═══════════════════════════════════════════════════════════════

const FULL_SNAPSHOT: &str = r#"
use sea_orm_migration::prelude::*;
pub struct Migration;
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("articles"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("id")).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Alias::new("title")).string_len(50).not_null())
                    .col(ColumnDef::new(Alias::new("views")).integer().not_null().default(0))
                    .col(ColumnDef::new_with_type(Alias::new("status"), ColumnType::Enum { name: Alias::new("Status").into_iden(), variants: vec![Alias::new("Draft").into_iden(), Alias::new("Published").into_iden()] }).not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).date_time().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Alias::new("author_id")).integer().not_null())
                    .to_owned()
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("articles_author_id_eihwaz_users_fkey")
                    .from(Alias::new("articles"), Alias::new("author_id"))
                    .to(Alias::new("eihwaz_users"), Alias::new("id"))
                    .on_delete(ForeignKeyAction::Restrict)
                    .on_update(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("articles_title_author_id_uniq")
                    .table(Alias::new("articles"))
                    .col(Alias::new("title"))
                    .col(Alias::new("author_id"))
                    .unique()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> { Ok(()) }
}
"#;

fn col<'a>(
    s: &'a runique::migration::utils::types::ParsedSchema,
    name: &str,
) -> &'a runique::migration::utils::types::ParsedColumn {
    s.columns
        .iter()
        .find(|c| c.name == name)
        .unwrap_or_else(|| panic!("column {name}"))
}

#[test]
fn test_full_snapshot_reads_back_every_detail() {
    let s = parse_seaorm_source(FULL_SNAPSHOT).expect("parse");
    assert_eq!(s.table_name, "articles");

    // An index naming `title` must not add a second, type-less `title` column
    // (the diff builds a map from this list: the last one would win).
    let mut names: Vec<&str> = s.columns.iter().map(|c| c.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        ["author_id", "created_at", "status", "title", "views"]
    );
    let title = col(&s, "title");
    assert_eq!(
        (title.col_type.as_str(), title.max_length),
        ("String", Some(50))
    );

    let views = col(&s, "views");
    assert_eq!(views.default_value.as_deref(), Some("0"));
    assert!(!views.has_default_now);

    let created = col(&s, "created_at");
    assert!(created.has_default_now && created.default_value.is_none());

    let status = col(&s, "status");
    assert_eq!(status.enum_name.as_deref(), Some("Status"));
    assert_eq!(status.enum_string_values, ["Draft", "Published"]);

    let fk = s.foreign_keys.first().expect("fk");
    assert_eq!(
        (
            fk.from_column.as_str(),
            fk.to_table.as_str(),
            fk.to_column.as_str()
        ),
        ("author_id", "eihwaz_users", "id")
    );
    assert_eq!(
        (fk.on_delete.as_str(), fk.on_update.as_str()),
        ("Restrict", "SetNull")
    );

    let idx = s.indexes.first().expect("index");
    assert_eq!(idx.name, "articles_title_author_id_uniq");
    assert_eq!(idx.columns, ["title", "author_id"]);
    assert!(idx.unique);
}
