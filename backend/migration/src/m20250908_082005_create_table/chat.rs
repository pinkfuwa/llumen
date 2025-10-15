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
                    .table(Chat::Table)
                    .if_not_exists()
                    .col(pk_auto(Chat::Id))
                    .col(integer(Chat::OwnerId))
                    .col(integer(Chat::Mode))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat-owner_id-user")
                            .from(Chat::Table, Chat::OwnerId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer_null(Chat::ModelId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat-model_id-model")
                            .from(Chat::Table, Chat::ModelId)
                            .to(Model::Table, Model::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .col(string_null(Chat::Title))
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-chat-owner_id-id")
                    .table(Chat::Table)
                    .col(Chat::OwnerId)
                    .col(Chat::Id)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-chat-owner_id-id")
                    .table(Chat::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Chat::Table).to_owned())
            .await?;

        Ok(())
    }
}
