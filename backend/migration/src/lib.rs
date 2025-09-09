pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Database;

mod m20250908_082005_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250908_082005_create_table::Migration)]
    }
}

pub async fn migrate(database_url: &str) -> Result<(), DbErr> {
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await
}
