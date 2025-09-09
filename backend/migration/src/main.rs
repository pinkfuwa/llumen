use sea_orm_migration::prelude::*;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    cli::run_cli(migration::Migrator).await;
}
