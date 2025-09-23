use sea_orm::{DbErr, Statement};
use sea_orm_migration::{prelude::*, sea_orm::DbBackend};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        if !matches!(backend, DbBackend::Sqlite) {
            return Ok(());
        }

        let stmt = Statement::from_string(backend, "PRAGMA journal_mode=WAL;".to_owned());
        manager.get_connection().execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let backend = manager.get_database_backend();

        if !matches!(backend, DbBackend::Sqlite) {
            return Ok(());
        }

        let stmt = Statement::from_string(backend, "PRAGMA journal_mode=DELETE;".to_owned());
        manager.get_connection().execute(stmt).await?;

        Ok(())
    }
}
