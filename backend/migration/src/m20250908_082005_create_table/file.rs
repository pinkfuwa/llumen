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
                    .table(File::Table)
                    .if_not_exists()
                    .col(pk_auto(File::Id))
                    .col(integer(File::MessageId))
                    .col(string(File::Name))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file-message_id-message")
                            .from(File::Table, File::MessageId)
                            .to(Message::Table, Message::Id)
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
                    .name("idx-file-message_id")
                    .table(File::Table)
                    .col(File::MessageId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-file-message_id")
                    .table(File::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await?;

        Ok(())
    }
}