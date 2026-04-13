//! Fournit les statements de création de tables SeaORM pour les composants RBAC internes.
//! Utilisable par les développeurs dans leurs propres migrations `up`.

use sea_query::{
    Alias, ColumnDef, ForeignKey, ForeignKeyAction, Index, Table, TableCreateStatement,
};

// ── EihwazUsersMigration ──────────────────────────────────────────────────────

/// Génère le `TableCreateStatement` pour la table `eihwaz_users`.
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

/// Migration "clé en main" pour créer la table `eihwaz_users`.
/// À placer en premier dans le `vec!` de `Migrator`.
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

/// Génère le `TableCreateStatement` pour la table `eihwaz_groupes`.
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

/// Génère le `TableCreateStatement` pour la table `eihwaz_groupes_droits`.
/// Chaque ligne représente les permissions d'un groupe sur une ressource spécifique.
/// PK composite : (groupe_id, resource_key).
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

/// Retourne le nom de la table user configurée.
/// Lit `RUNIQUE_USER_TABLE` depuis l'environnement (`.env` chargé par sea-orm-cli).
/// Défaut : `"eihwaz_users"`.
pub fn user_table_name() -> String {
    std::env::var("RUNIQUE_USER_TABLE").unwrap_or_else(|_| "eihwaz_users".to_string())
}

/// Génère le `TableCreateStatement` pour la table pivot `eihwaz_users_groupes`.
/// La FK vers la table user cible `RUNIQUE_USER_TABLE` (défaut : `eihwaz_users`).
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

/// Migration complète "clé en main" pour initialiser l'architecture RBAC native de Runique.
/// À injecter directement dans le `vec!` de `Migrations::up()` après la migration de votre table User.
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
        Ok(())
    }

    async fn down(&self, manager: &sea_orm_migration::SchemaManager) -> Result<(), sea_orm::DbErr> {
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
