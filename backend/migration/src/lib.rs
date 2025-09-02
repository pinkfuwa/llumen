pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Database;

mod m20250724_144358_create_tables;
mod m20250824_044739_add_user_config;
mod m20250828_123532_add_default_model;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250724_144358_create_tables::Migration),
            Box::new(m20250824_044739_add_user_config::Migration),
            Box::new(m20250828_123532_add_default_model::Migration),
        ]
    }
}

pub async fn migrate(database_url: &str) -> Result<(), DbErr> {
    let db = Database::connect(database_url).await?;
    Migrator::up(&db, None).await
}
