use sea_orm_migration::prelude::*;

// Reads DATABASE_URL from the project's `.env`: run it from the project root,
// through `runique migration up | down | status`.
#[tokio::main]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
