use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(runique_cli::migration::runtime::RuniqueMigrator(migration::Migrator)).await;
}
