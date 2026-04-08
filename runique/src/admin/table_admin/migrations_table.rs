//! Fournit les statements de création de tables SeaORM pour les composants RBAC internes.
//! Utilisable par les développeurs dans leurs propres migrations `up`.

use sea_query::{
    Alias, ColumnDef, ForeignKey, ForeignKeyAction, Index, Table, TableCreateStatement,
};

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

/// Génère le `TableCreateStatement` pour la table `eihwaz_droits` (la matrice de permissions).
pub fn create_eihwaz_droits_table() -> TableCreateStatement {
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
        .index(
            Index::create()
                .name("uq_eihwaz_droits_groupe_resource")
                .col(Alias::new("groupe_id"))
                .col(Alias::new("resource_key"))
                .unique(),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_eihwaz_droits_groupe_id")
                .from(Alias::new("eihwaz_droits"), Alias::new("groupe_id"))
                .to(Alias::new("eihwaz_groupes"), Alias::new("id"))
                .on_delete(ForeignKeyAction::Cascade)
                .on_update(ForeignKeyAction::Cascade),
        )
        .to_owned()
}

/// Génère le `TableCreateStatement` pour la table pivot `eihwaz_users_groupes`.
pub fn create_eihwaz_users_groupes_table() -> TableCreateStatement {
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
                .name("fk_eihwaz_users_groupes_user_id")
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
        manager.create_table(create_eihwaz_droits_table()).await?;
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
            .drop_table(Table::drop().table(Alias::new("eihwaz_droits")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("eihwaz_groupes")).to_owned())
            .await?;
        Ok(())
    }
}
