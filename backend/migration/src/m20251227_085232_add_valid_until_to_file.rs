use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(File::Table)
                    .add_column(integer_null(File::ValidUntil))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(Index::drop().name("idx-file-chat_id").to_owned())
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-file-chat_id-valid_until")
                    .table(File::Table)
                    .col(File::ChatId)
                    .col(File::ValidUntil)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-file-chat_id-valid_until")
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-file-chat_id")
                    .table(File::Table)
                    .col(File::ChatId)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(File::Table)
                    .drop_column(File::ValidUntil)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum File {
    Table,
    ChatId,
    ValidUntil,
}
