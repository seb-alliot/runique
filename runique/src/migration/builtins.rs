/// Migrations built-in Runique — tables infrastructure `eihwaz_*`
///
/// À utiliser dans le `lib.rs` du crate migration du projet :
///
/// ```rust,ignore
/// fn migrations() -> Vec<Box<dyn MigrationTrait>> {
///     let mut m = vec![
///         Box::new(m20260327_..._create_eihwaz_users_table::Migration), // géré par makemigrations
///         // ...autres tables du projet...
///     ];
///     m.extend(runique::migration::builtin_migrations());
///     m
/// }
/// ```
///
/// **Important :** `eihwaz_users` doit apparaître **avant** les builtin_migrations
/// car sessions et permissions ont une FK vers cette table.
use sea_orm_migration::prelude::*;

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_sessions
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazSessions;

impl MigrationName for CreateEihwazSessions {
    fn name(&self) -> &str {
        "m20000101_000001_create_eihwaz_sessions"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazSessions {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_sessions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("session_id"))
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("session_id_user"))
                            .string()
                            .null(),
                    )
                    .col(ColumnDef::new(Alias::new("user_id")).integer().null())
                    .col(ColumnDef::new(Alias::new("device")).string().null())
                    .col(ColumnDef::new(Alias::new("data")).json_binary().not_null())
                    .col(
                        ColumnDef::new(Alias::new("expires_at"))
                            .date_time()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_sessions"), Alias::new("user_id"))
                            .to(Alias::new("eihwaz_users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Alias::new("eihwaz_sessions"))
                    .name("idx_eihwaz_sessions_user_id")
                    .col(Alias::new("user_id"))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .table(Alias::new("eihwaz_sessions"))
                    .name("idx_eihwaz_sessions_expires_at")
                    .col(Alias::new("expires_at"))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_sessions"))
                    .to_owned(),
            )
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_droits
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazDroits;

impl MigrationName for CreateEihwazDroits {
    fn name(&self) -> &str {
        "m20000101_000002_create_eihwaz_droits"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazDroits {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_droits"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("nom"))
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_droits")).to_owned())
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_groupes
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazGroupes;

impl MigrationName for CreateEihwazGroupes {
    fn name(&self) -> &str {
        "m20000101_000003_create_eihwaz_groupes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazGroupes {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_groupes"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Alias::new("nom"))
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_groupes")).to_owned())
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_users_droits  (junction users ↔ droits)
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazUsersDroits;

impl MigrationName for CreateEihwazUsersDroits {
    fn name(&self) -> &str {
        "m20000101_000004_create_eihwaz_users_droits"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazUsersDroits {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_users_droits"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("droit_id")).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alias::new("user_id"))
                            .col(Alias::new("droit_id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_users_droits"), Alias::new("user_id"))
                            .to(Alias::new("eihwaz_users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_users_droits"), Alias::new("droit_id"))
                            .to(Alias::new("eihwaz_droits"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_users_droits"))
                    .to_owned(),
            )
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_users_groupes  (junction users ↔ groupes)
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazUsersGroupes;

impl MigrationName for CreateEihwazUsersGroupes {
    fn name(&self) -> &str {
        "m20000101_000005_create_eihwaz_users_groupes"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazUsersGroupes {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_users_groupes"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("groupe_id")).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alias::new("user_id"))
                            .col(Alias::new("groupe_id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_users_groupes"), Alias::new("user_id"))
                            .to(Alias::new("eihwaz_users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_users_groupes"), Alias::new("groupe_id"))
                            .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_users_groupes"))
                    .to_owned(),
            )
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// eihwaz_groupes_droits  (junction groupes ↔ droits)
// ══════════════════════════════════════════════════════════════════════════════

pub struct CreateEihwazGroupesDroits;

impl MigrationName for CreateEihwazGroupesDroits {
    fn name(&self) -> &str {
        "m20000101_000006_create_eihwaz_groupes_droits"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CreateEihwazGroupesDroits {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("eihwaz_groupes_droits"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("groupe_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("droit_id")).integer().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alias::new("groupe_id"))
                            .col(Alias::new("droit_id")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_groupes_droits"), Alias::new("groupe_id"))
                            .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Alias::new("eihwaz_groupes_droits"), Alias::new("droit_id"))
                            .to(Alias::new("eihwaz_droits"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_groupes_droits"))
                    .to_owned(),
            )
            .await
    }
}

// ══════════════════════════════════════════════════════════════════════════════
// Point d'entrée public
// ══════════════════════════════════════════════════════════════════════════════

/// Retourne les migrations built-in Runique (sessions + permissions).
///
/// À insérer **après** la migration `eihwaz_users` dans le vecteur du projet.
/// Retourne les migrations built-in Runique.
///
/// - `eihwaz_sessions` : store de sessions DB
/// - Tables de jonction permissions (`eihwaz_users_droits`, `eihwaz_users_groupes`, `eihwaz_groupes_droits`)
///
/// `eihwaz_users`, `eihwaz_droits` et `eihwaz_groupes` sont gérés par `makemigrations`
/// (entités dans `src/entities/`) et doivent apparaître **avant** dans le `vec!`.
pub fn builtin_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(CreateEihwazSessions),
        Box::new(CreateEihwazUsersDroits),
        Box::new(CreateEihwazUsersGroupes),
        Box::new(CreateEihwazGroupesDroits),
    ]
}

// Silence l'avertissement sur DbBackend importé mais non utilisé si SQLite-only
#[allow(unused_imports)]
use sea_orm::DbBackend as _;
