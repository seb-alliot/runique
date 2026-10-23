use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("test_relation_child"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("parent_id")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("label")).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("test_relation_child_parent_id_test_relation_parent_fkey")
                            .from(Alias::new("test_relation_child"), Alias::new("parent_id"))
                            .to(Alias::new("test_relation_parent"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::NoAction),
                    )
                    .to_owned(),
            )
            .await?;

        {
            let backs_fk = manager.get_connection().get_database_backend()
                == sea_orm::DbBackend::MySql
                && manager
                    .get_connection()
                    .query_one(
                        &Query::select()
                            .expr(Expr::val(1))
                            .from((
                                Alias::new("information_schema"),
                                Alias::new("KEY_COLUMN_USAGE"),
                            ))
                            .and_where(
                                Expr::col(Alias::new("TABLE_SCHEMA")).eq(Expr::cust("DATABASE()")),
                            )
                            .and_where(
                                Expr::col(Alias::new("TABLE_NAME")).eq("test_relation_child"),
                            )
                            .and_where(Expr::col(Alias::new("COLUMN_NAME")).eq("parent_id"))
                            .and_where(Expr::col(Alias::new("REFERENCED_TABLE_NAME")).is_not_null())
                            .to_owned(),
                    )
                    .await?
                    .is_some();
            if !backs_fk {
                manager
                    .create_index(
                        Index::create()
                            .name("idx_test_relation_child_parent_id")
                            .table(Alias::new("test_relation_child"))
                            .col(Alias::new("parent_id"))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("test_relation_child"))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
