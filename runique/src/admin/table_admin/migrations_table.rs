//! Provides SeaORM table creation statements for internal RBAC components.
//! Usable by developers in their own `up` migrations.

use sea_query::{
    Alias, ColumnDef, ForeignKey, ForeignKeyAction, Index, Table, TableCreateStatement,
};

// ── EihwazUsersMigration ──────────────────────────────────────────────────────

/// Generates the `TableCreateStatement` for the `eihwaz_users` table.
pub fn create_eihwaz_users_table() -> TableCreateStatement {
    let mut pk_col = ColumnDef::new(Alias::new("id"));
    #[cfg(feature = "big-pk")]
    pk_col.big_integer();
    #[cfg(not(feature = "big-pk"))]
    pk_col.integer();
    pk_col.not_null().auto_increment().primary_key();

    Table::create()
        .table(Alias::new("eihwaz_users"))
        .if_not_exists()
        .col(&mut pk_col)
        .col(
            ColumnDef::new(Alias::new("username"))
                .string()
                .not_null()
                .unique_key(),
        )
        .col(
            ColumnDef::new(Alias::new("email"))
                .string()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(Alias::new("password")).string().not_null())
        .col(
            ColumnDef::new(Alias::new("is_active"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("is_staff"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("is_superuser"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(ColumnDef::new(Alias::new("created_at")).date_time().null())
        .col(ColumnDef::new(Alias::new("updated_at")).date_time().null())
        .to_owned()
}

/// "Turnkey" migration to create the `eihwaz_users` table.
/// To be placed first in the `Migrator` `vec!`.
pub struct EihwazUsersMigration;

impl sea_orm_migration::MigrationName for EihwazUsersMigration {
    fn name(&self) -> &str {
        "m000000_000001_runique_eihwaz_users"
    }
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for EihwazUsersMigration {
    async fn up(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager.create_table(create_eihwaz_users_table()).await
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_users")).to_owned())
            .await
    }
}

/// Generates the `TableCreateStatement` for the `eihwaz_groupes` table.
pub fn create_eihwaz_groupes_table() -> TableCreateStatement {
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
        .to_owned()
}

/// Generates the `TableCreateStatement` for the `eihwaz_groupes_droits` table.
/// Each row represents a group's permissions on a specific resource.
/// Composite PK: (groupe_id, resource_key).
pub fn create_eihwaz_groupes_droits_table() -> TableCreateStatement {
    Table::create()
        .table(Alias::new("eihwaz_groupes_droits"))
        .if_not_exists()
        .col(ColumnDef::new(Alias::new("groupe_id")).integer().not_null())
        .col(
            ColumnDef::new(Alias::new("resource_key"))
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("can_create"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("can_read"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("can_update"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("can_delete"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("can_update_own"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("can_delete_own"))
                .boolean()
                .not_null()
                .default(false),
        )
        .primary_key(
            Index::create()
                .name("pk_eihwaz_groupes_droits")
                .col(Alias::new("groupe_id"))
                .col(Alias::new("resource_key")),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_eihwaz_groupes_droits_groupe_id")
                .from(Alias::new("eihwaz_groupes_droits"), Alias::new("groupe_id"))
                .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// Returns the name of the configured user table.
/// Reads `RUNIQUE_USER_TABLE` from the environment (`.env` loaded by `sea-orm-cli`).
/// Default: `"eihwaz_users"`.
pub fn user_table_name() -> String {
    std::env::var("RUNIQUE_USER_TABLE").unwrap_or_else(|_| "eihwaz_users".to_string())
}

/// Generates the `TableCreateStatement` for the `eihwaz_users_groupes` junction table.
/// The FK to the user table targets `RUNIQUE_USER_TABLE` (default: `eihwaz_users`).
pub fn create_eihwaz_users_groupes_table() -> TableCreateStatement {
    let user_table = user_table_name();
    let fk_name = format!("fk_eihwaz_users_groupes_{}_id", user_table);

    Table::create()
        .table(Alias::new("eihwaz_users_groupes"))
        .if_not_exists()
        .col(ColumnDef::new(Alias::new("user_id")).integer().not_null())
        .col(ColumnDef::new(Alias::new("groupe_id")).integer().not_null())
        .primary_key(
            Index::create()
                .name("pk_eihwaz_users_groupes")
                .col(Alias::new("user_id"))
                .col(Alias::new("groupe_id")),
        )
        .foreign_key(
            ForeignKey::create()
                .name(&fk_name)
                .from(Alias::new("eihwaz_users_groupes"), Alias::new("user_id"))
                .to(Alias::new(user_table.as_str()), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_eihwaz_users_groupes_groupe_id")
                .from(Alias::new("eihwaz_users_groupes"), Alias::new("groupe_id"))
                .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// Generates the `TableCreateStatement` for the `eihwaz_sessions` table.
pub fn create_eihwaz_sessions_table() -> TableCreateStatement {
    let mut user_id_col = ColumnDef::new(Alias::new("user_id"));
    #[cfg(feature = "big-pk")]
    user_id_col.big_integer();
    #[cfg(not(feature = "big-pk"))]
    user_id_col.integer();
    user_id_col.not_null();

    let user_table = user_table_name();
    let fk_name = format!("fk_eihwaz_sessions_{}_id", user_table);

    Table::create()
        .table(Alias::new("eihwaz_sessions"))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("cookie_id"))
                .string()
                .not_null()
                .unique_key(),
        )
        .col(&mut user_id_col)
        .col(ColumnDef::new(Alias::new("session_id")).string().not_null())
        .col(ColumnDef::new(Alias::new("session_data")).text().null())
        .col(
            ColumnDef::new(Alias::new("expires_at"))
                .date_time()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name(&fk_name)
                .from(Alias::new("eihwaz_sessions"), Alias::new("user_id"))
                .to(Alias::new(user_table.as_str()), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// "Turnkey" migration to create the `eihwaz_sessions` table.
/// To be placed in the `Migrator` `vec!` after `EihwazUsersMigration`.
pub struct EihwazSessionsMigration;

impl sea_orm_migration::MigrationName for EihwazSessionsMigration {
    fn name(&self) -> &str {
        "m000000_000003_runique_eihwaz_sessions"
    }
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for EihwazSessionsMigration {
    async fn up(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager.create_table(create_eihwaz_sessions_table()).await
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_sessions"))
                    .to_owned(),
            )
            .await
    }
}

/// Generates the `TableCreateStatement` for the `eihwaz_history` table.
pub fn create_eihwaz_history_table() -> TableCreateStatement {
    let mut user_id_col = ColumnDef::new(Alias::new("user_id"));
    #[cfg(feature = "big-pk")]
    user_id_col.big_integer();
    #[cfg(not(feature = "big-pk"))]
    user_id_col.integer();
    user_id_col.not_null();

    Table::create()
        .table(Alias::new("eihwaz_history"))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new("id"))
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("resource_key"))
                .string()
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("object_pk")).string().not_null())
        .col(ColumnDef::new(Alias::new("action")).string().not_null())
        .col(&mut user_id_col)
        .col(ColumnDef::new(Alias::new("username")).string().not_null())
        .col(
            ColumnDef::new(Alias::new("created_at"))
                .date_time()
                .not_null(),
        )
        .col(ColumnDef::new(Alias::new("summary")).text().null())
        .col(ColumnDef::new(Alias::new("batch_id")).string().null())
        .to_owned()
}

/// "Turnkey" migration to create the `eihwaz_history` table.
pub struct EihwazHistoryMigration;

impl sea_orm_migration::MigrationName for EihwazHistoryMigration {
    fn name(&self) -> &str {
        "m000000_000004_runique_eihwaz_history"
    }
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for EihwazHistoryMigration {
    async fn up(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager.create_table(create_eihwaz_history_table()).await
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_history")).to_owned())
            .await
    }
}

/// Complete "turnkey" migration to initialize Runique's native RBAC architecture.
/// To be injected directly into the `Migrations::up()` `vec!` after your User table migration.
pub struct AdminTableMigration;

impl sea_orm_migration::MigrationName for AdminTableMigration {
    fn name(&self) -> &str {
        "m000000_000002_runique_admin_table"
    }
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for AdminTableMigration {
    async fn up(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager.create_table(create_eihwaz_groupes_table()).await?;
        manager
            .create_table(create_eihwaz_groupes_droits_table())
            .await?;
        manager
            .create_table(create_eihwaz_users_groupes_table())
            .await?;
        manager.create_table(create_eihwaz_history_table()).await?;
        Ok(())
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_history")).to_owned())
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_users_groupes"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_groupes_droits"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_groupes")).to_owned())
            .await?;
        Ok(())
    }
}
