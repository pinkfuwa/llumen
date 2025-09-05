use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Tool::Table)
                    .if_not_exists()
                    .col(integer(Tool::ChatId))
                    .col(string(Tool::FunctionName))
                    .col(string(Tool::State))
                    .primary_key(Index::create().col(Tool::ChatId).col(Tool::FunctionName))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Tool::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Tool {
    Table,
    ChatId,
    FunctionName,
    State,
}
