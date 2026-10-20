//! The framework's own tables (accounts, groups, rights, sessions, reset
//! tokens), needed with or without the admin. Usable by developers in their
//! own `up` migrations. The admin's history table is created here too; only
//! its use (entity, pages) belongs to the admin.

use crate::utils::constante::resource_key::FRAMEWORK_KEYS;
use sea_query::{
    Alias, Check, ColumnDef, Expr, ExprTrait, ForeignKey, ForeignKeyAction, Index,
    IndexCreateStatement, Table, TableCreateStatement,
};

/// A column pointing at `eihwaz_users.id`, of the same type: a FK of another
/// type fails on Postgres.
fn user_pk_reference(name: &str) -> ColumnDef {
    let mut col = ColumnDef::new(Alias::new(name));
    #[cfg(feature = "pk-uuid")]
    col.uuid();
    #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
    col.big_integer();
    #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
    col.integer();
    col
}

// ── EihwazUsersMigration ──────────────────────────────────────────────────────

/// Generates the `TableCreateStatement` for the `eihwaz_users` table.
pub fn create_eihwaz_users_table() -> TableCreateStatement {
    let mut pk_col = ColumnDef::new(Alias::new("id"));
    #[cfg(feature = "pk-uuid")]
    pk_col.uuid();
    #[cfg(all(feature = "big-pk", not(feature = "pk-uuid")))]
    pk_col.big_integer();
    #[cfg(not(any(feature = "big-pk", feature = "pk-uuid")))]
    pk_col.integer();
    pk_col.not_null().primary_key();
    // UUID primary keys are generated application-side (Uuid::now_v7()), never DB auto-increment.
    #[cfg(not(feature = "pk-uuid"))]
    pk_col.auto_increment();

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
            ColumnDef::new(Alias::new("is_dirigeant"))
                .boolean()
                .not_null()
                .default(false),
        )
        .col(
            ColumnDef::new(Alias::new("is_admin"))
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
        // When the account's owner first took it over through the emailed link.
        .col(
            ColumnDef::new(Alias::new("activated_at"))
                .date_time()
                .null(),
        )
        // The database itself refuses an active account that was never
        // activated, whatever writes it — the ORM, raw SQL, a script.
        .check(Check::named(
            ACTIVE_NEEDS_ACTIVATION,
            Expr::col(Alias::new("is_active"))
                .not()
                .or(Expr::col(Alias::new("activated_at")).is_not_null()),
        ))
        // A boolean isn't a number on every engine: CASE turns each rank flag
        // into 0/1 the same way on the three.
        .check(Check::named(
            ONE_RANK_AT_MOST,
            rank_flag("is_superuser")
                .add(rank_flag("is_dirigeant"))
                .add(rank_flag("is_admin"))
                .lte(1),
        ))
        .to_owned()
}

fn rank_flag(column: &str) -> Expr {
    Expr::case(Expr::col(Alias::new(column)), 1)
        .finally(0)
        .into()
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

/// Name of the `CHECK` constraint "an active account has been activated", so a
/// refusal can be recognised whatever the engine.
pub const ACTIVE_NEEDS_ACTIVATION: &str = "eihwaz_users_active_needs_activation";

/// Name of the `CHECK` constraint "at most one of superuser, dirigeant, admin".
pub const ONE_RANK_AT_MOST: &str = "eihwaz_users_one_rank_at_most";

/// Name of the `CHECK` "a row belongs to exactly one group or one account".
pub const DROITS_ONE_HOLDER: &str = "eihwaz_groupes_droits_one_holder";

/// Name of the `CHECK` "a group's row never targets a framework table".
pub const DROITS_GROUP_NEVER_FRAMEWORK: &str = "eihwaz_groupes_droits_group_never_framework";

/// Name of the `CHECK` "a column row only reads, creates or updates".
pub const DROITS_COLUMN_ROW_FLAGS: &str = "eihwaz_groupes_droits_column_row_flags";

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
        .col(
            ColumnDef::new(Alias::new("is_staff"))
                .boolean()
                .not_null()
                .default(false),
        )
        .to_owned()
}

/// Generates the `TableCreateStatement` for the `eihwaz_groupes_droits` table:
/// the rights of a group (site's tables) or of an admin account, on a whole
/// table (`colonne = ''`) or on one of its columns.
pub fn create_eihwaz_groupes_droits_table() -> TableCreateStatement {
    let flag = |name: &str| {
        ColumnDef::new(Alias::new(name))
            .boolean()
            .not_null()
            .default(false)
            .to_owned()
    };
    let col = |name: &str| Expr::col(Alias::new(name));
    let mut user_id_col = user_pk_reference("user_id");
    user_id_col.null();

    Table::create()
        .table(Alias::new("eihwaz_groupes_droits"))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new("id"))
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Alias::new("groupe_id")).integer().null())
        .col(&mut user_id_col)
        .col(
            ColumnDef::new(Alias::new("resource_key"))
                .string()
                .not_null(),
        )
        .col(
            ColumnDef::new(Alias::new("colonne"))
                .string()
                .not_null()
                .default(""),
        )
        .col(flag("can_create"))
        .col(flag("can_read"))
        .col(flag("can_read_own"))
        .col(flag("can_update"))
        .col(flag("can_update_own"))
        .col(flag("can_delete"))
        .col(flag("can_delete_own"))
        .check(Check::named(
            DROITS_ONE_HOLDER,
            col("groupe_id")
                .is_null()
                .and(col("user_id").is_not_null())
                .or(col("groupe_id").is_not_null().and(col("user_id").is_null())),
        ))
        .check(Check::named(
            DROITS_GROUP_NEVER_FRAMEWORK,
            col("groupe_id")
                .is_null()
                .or(col("resource_key").is_not_in(FRAMEWORK_KEYS.iter().copied())),
        ))
        .check(Check::named(
            DROITS_COLUMN_ROW_FLAGS,
            col("colonne").eq("").or(col("can_read_own")
                .not()
                .and(col("can_update_own").not())
                .and(col("can_delete").not())
                .and(col("can_delete_own").not())),
        ))
        .foreign_key(
            ForeignKey::create()
                .name("fk_eihwaz_groupes_droits_groupe_id")
                .from(Alias::new("eihwaz_groupes_droits"), Alias::new("groupe_id"))
                .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_eihwaz_groupes_droits_user_id")
                .from(Alias::new("eihwaz_groupes_droits"), Alias::new("user_id"))
                .to(Alias::new("eihwaz_users"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// One row per (group, table, column). NULL `groupe_id`s (account rows) are
/// distinct from each other on the three engines, so they never collide here.
pub fn create_droits_unique_par_groupe() -> IndexCreateStatement {
    Index::create()
        .name("uq_eihwaz_groupes_droits_groupe")
        .table(Alias::new("eihwaz_groupes_droits"))
        .col(Alias::new("groupe_id"))
        .col(Alias::new("resource_key"))
        .col(Alias::new("colonne"))
        .unique()
        .to_owned()
}

/// One row per (account, table, column).
pub fn create_droits_unique_par_compte() -> IndexCreateStatement {
    Index::create()
        .name("uq_eihwaz_groupes_droits_user")
        .table(Alias::new("eihwaz_groupes_droits"))
        .col(Alias::new("user_id"))
        .col(Alias::new("resource_key"))
        .col(Alias::new("colonne"))
        .unique()
        .to_owned()
}

/// Generates the `TableCreateStatement` for the `eihwaz_users_groupes` junction table.
pub fn create_eihwaz_users_groupes_table() -> TableCreateStatement {
    let fk_name = "fk_eihwaz_users_groupes_eihwaz_users_id";

    // user_id doit suivre le type de eihwaz_users.id (BIGINT sous big-pk, UUID sous pk-uuid),
    // sinon la FK est de type incompatible et échoue à la création (Postgres strict).
    let mut user_id_col = user_pk_reference("user_id");
    user_id_col.not_null();

    Table::create()
        .table(Alias::new("eihwaz_users_groupes"))
        .if_not_exists()
        .col(&mut user_id_col)
        .col(ColumnDef::new(Alias::new("groupe_id")).integer().not_null())
        .primary_key(
            Index::create()
                .name("pk_eihwaz_users_groupes")
                .col(Alias::new("user_id"))
                .col(Alias::new("groupe_id")),
        )
        .foreign_key(
            ForeignKey::create()
                .name(fk_name)
                .from(Alias::new("eihwaz_users_groupes"), Alias::new("user_id"))
                .to(Alias::new("eihwaz_users"), Alias::new("id"))
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
    let mut user_id_col = user_pk_reference("user_id");
    user_id_col.not_null();

    let fk_name = "fk_eihwaz_sessions_eihwaz_users_id";

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
                .name(fk_name)
                .from(Alias::new("eihwaz_sessions"), Alias::new("user_id"))
                .to(Alias::new("eihwaz_users"), Alias::new("id"))
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
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

// The admin's history: its migration stays here, always compiled, while the
// table is only used with the `admin` feature. Once a database has applied it,
// SeaORM wants it in the `Migrator` for good, admin or not.

/// Generates the `TableCreateStatement` for the `eihwaz_history` table.
///
/// `user_id` **deliberately has no FK** to the user table: the audit trail must
/// survive a user's deletion (a CASCADE FK would erase the history of their
/// actions). `username` is denormalized to keep a readable trace even after
/// deletion. Don't "fix" this by adding a CASCADE FK.
pub fn create_eihwaz_history_table() -> TableCreateStatement {
    let mut user_id_col = user_pk_reference("user_id");
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

/// Generates the `TableCreateStatement` for the `eihwaz_reset_tokens` table.
///
/// Stores **hashed** password-reset tokens (never the raw token) so a DB read leak
/// cannot be replayed. `user_id` is the authoritative target of the reset, derived
/// from the token row — never from a URL/form field.
pub fn create_eihwaz_reset_tokens_table() -> TableCreateStatement {
    let mut user_id_col = user_pk_reference("user_id");
    user_id_col.not_null();

    let fk_name = "fk_eihwaz_reset_tokens_eihwaz_users_id";

    Table::create()
        .table(Alias::new("eihwaz_reset_tokens"))
        .if_not_exists()
        .col(
            ColumnDef::new(Alias::new("id"))
                .big_integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(Alias::new("token_hash"))
                .string()
                .not_null()
                .unique_key(),
        )
        .col(&mut user_id_col)
        .col(
            ColumnDef::new(Alias::new("expires_at"))
                .date_time()
                .not_null(),
        )
        .foreign_key(
            ForeignKey::create()
                .name(fk_name)
                .from(Alias::new("eihwaz_reset_tokens"), Alias::new("user_id"))
                .to(Alias::new("eihwaz_users"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// "Turnkey" migration to create the `eihwaz_reset_tokens` table.
/// To be placed in the `Migrator` `vec!` after `EihwazUsersMigration`.
pub struct EihwazResetTokensMigration;

impl sea_orm_migration::MigrationName for EihwazResetTokensMigration {
    fn name(&self) -> &str {
        "m000000_000005_runique_reset_tokens"
    }
}

#[async_trait::async_trait]
impl sea_orm_migration::MigrationTrait for EihwazResetTokensMigration {
    async fn up(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .create_table(create_eihwaz_reset_tokens_table())
            .await
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_reset_tokens"))
                    .if_exists()
                    .to_owned(),
            )
            .await
    }
}

/// Complete "turnkey" migration to initialize Runique's native RBAC architecture.
/// To be injected directly into the `Migrations::up()` `vec!` after your User table migration.
///
/// It used to create `eihwaz_history` too: [`EihwazHistoryMigration`] does it now,
/// injected by `makemigrations` only when the project has the admin.
/// The name below is recorded in every existing database: never change it.
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
            .create_index(create_droits_unique_par_groupe())
            .await?;
        manager
            .create_index(create_droits_unique_par_compte())
            .await?;
        manager
            .create_table(create_eihwaz_users_groupes_table())
            .await?;
        manager.create_table(create_eihwaz_sessions_table()).await?;
        Ok(())
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("eihwaz_sessions"))
                    .if_exists()
                    .to_owned(),
            )
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
