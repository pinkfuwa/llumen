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
                    .table(Chunk::Table)
                    .col(pk_auto(Chunk::Id))
                    .col(string_null(Chunk::Content))
                    .col(integer(Chunk::Kind))
                    .col(integer(Chunk::MessageId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chunk-message_id-message")
                            .from(Chunk::Table, Chunk::MessageId)
                            .to(Message::Table, Message::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chunk::Table).to_owned())
            .await?;

        Ok(())
    }
}
