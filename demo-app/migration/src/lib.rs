pub use sea_orm_migration::prelude::*;
mod blog;
mod eihwaz_users;
mod test_all_fields;
mod users_booster;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        // let mut migrations: Vec<Box<dyn MigrationTrait>> = user_runique()
        //     .into_iter()
        //     .collect();
        vec![
            Box::new(eihwaz_users::Migration),
            Box::new(blog::Migration),
            Box::new(users_booster::Migration),
            Box::new(test_all_fields::Migration),
        ]
    }
}
