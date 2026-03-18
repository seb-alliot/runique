pub use sea_orm_migration::prelude::*;
mod m20260318_140408_create_blog_table;
mod m20260318_140408_create_contributions_table;
mod m20260318_140408_create_eihwaz_users_table;
mod m20260318_140408_create_test_all_fields_table;
mod m20260318_140408_create_users_booster_table;
mod m20260318_140518_alter_contributions_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // let mut migrations: Vec<Box<dyn MigrationTrait>> = user_runique()
        //     .into_iter()
        //     .collect();
        vec![
            Box::new(m20260318_140408_create_blog_table::Migration),
            Box::new(m20260318_140408_create_contributions_table::Migration),
            Box::new(m20260318_140408_create_users_booster_table::Migration),
            Box::new(m20260318_140408_create_test_all_fields_table::Migration),
            Box::new(m20260318_140408_create_eihwaz_users_table::Migration),
            Box::new(m20260318_140518_alter_contributions_table::Migration),
        ]
    }
}
