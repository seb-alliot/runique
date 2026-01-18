use std::vec;

pub use sea_orm_migration::prelude::*;
mod m20260118_003649_create_users_table;
mod m20260118_003649_create_blog_table;
mod m20260118_003649_create_users_booster_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260118_003649_create_users_table::Migration),
            Box::new(m20260118_003649_create_blog_table::Migration),
            Box::new(m20260118_003649_create_users_booster_table::Migration),
        ]
    }
}

