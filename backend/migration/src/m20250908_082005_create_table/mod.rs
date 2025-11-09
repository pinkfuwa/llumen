use sea_orm_migration::prelude::*;

mod entity;

mod config;
mod model;
mod tool;
mod user;

mod chat;
mod file;
mod message;

mod default;
// WAL is not presistent across connections, however, we should flush wal
// mod wal;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250908_082005_create_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        config::Migration.up(manager).await?;
        tool::Migration.up(manager).await?;
        user::Migration.up(manager).await?;
        model::Migration.up(manager).await?;

        chat::Migration.up(manager).await?;
        message::Migration.up(manager).await?;

        default::Migration.up(manager).await?;
        file::Migration.up(manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        default::Migration.down(manager).await?;

        message::Migration.down(manager).await?;
        chat::Migration.down(manager).await?;
        model::Migration.down(manager).await?;
        user::Migration.down(manager).await?;
        config::Migration.down(manager).await?;
        tool::Migration.down(manager).await?;
        file::Migration.down(manager).await?;

        Ok(())
    }
}
