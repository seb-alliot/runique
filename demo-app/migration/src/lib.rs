pub use sea_orm_migration::prelude::*;

mod m20260118_003649_create_blog_table;
mod m20260118_003649_create_users_booster_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        let mut migrations = runique::migration::builtin_migrations();

        migrations.extend(vec![
            Box::new(m20260118_003649_create_blog_table::Migration) as Box<dyn MigrationTrait>,
            Box::new(m20260118_003649_create_users_booster_table::Migration)
                as Box<dyn MigrationTrait>,
        ]);

        migrations
    }
}
