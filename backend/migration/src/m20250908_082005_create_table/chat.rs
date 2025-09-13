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
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat-owner_id-user")
                            .from(Chat::Table, Chat::OwnerId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(integer(Chat::ModelId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat-model_id-model")
                            .from(Chat::Table, Chat::ModelId)
                            .to(Model::Table, Model::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string_null(Chat::Title))
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chat::Table).to_owned())
            .await?;

        Ok(())
    }
}
