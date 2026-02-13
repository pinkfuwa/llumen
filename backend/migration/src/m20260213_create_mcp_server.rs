use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(McpServer::Table)
                    .if_not_exists()
                    .col(pk_auto(McpServer::Id))
                    .col(string_uniq(McpServer::Name))
                    .col(text(McpServer::ConfigRaw))
                    .col(boolean(McpServer::Enabled).default(true))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(McpServer::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum McpServer {
    Table,
    Id,
    Name,
    ConfigRaw,
    Enabled,
}
