use sea_orm_migration::{prelude::*, schema::*};

use super::entity::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Message::Table)
                    .col(pk_auto(Message::Id))
                    .col(integer(Message::ChatId))
                    .col(float(Message::Price).default(0.0))
                    .col(integer(Message::TokenCount).default(0))
                    .col(string(Message::Inner))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-message-chat_id-chat")
                            .from(Message::Table, Message::ChatId)
                            .to(Chat::Table, Chat::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-message-chat_id")
                    .table(Message::Table)
                    .col(Message::ChatId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-message-chat_id")
                    .table(Message::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await?;

        Ok(())
    }
}
