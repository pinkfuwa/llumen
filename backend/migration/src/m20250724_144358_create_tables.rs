use sea_orm_migration::{manager, prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(pk_auto(User::Id))
                    .col(string(User::Name))
                    .col(string(User::Password))
                    .to_owned(),
            )
            .await?;

        // Set default admin user
        let default_admin = Query::insert()
            .into_table(User::Table)
            .columns([User::Name, User::Password])
            .values_panic(["admin".into(), "admin".into()])
            .to_owned();
        manager.exec_stmt(default_admin).await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-user-name")
                    .table(User::Table)
                    .col(User::Name)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Chat::Table)
                    .if_not_exists()
                    .col(integer(Chat::RoomId).primary_key())
                    .col(integer(Chat::OwnerId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-chat-owner_id-user")
                            .from(Chat::Table, Chat::OwnerId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(timestamp(Chat::CreatedAt))
                    .col(string(Chat::Text))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Model::Table)
                    .col(pk_auto(Model::Id))
                    .col(string(Model::Config))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Chat::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Model::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Name,
    Password,
}

#[derive(DeriveIden)]
enum Chat {
    Table,
    RoomId,
    OwnerId,
    CreatedAt,
    Text,
}

#[derive(DeriveIden)]
enum Model {
    Table,
    Id,
    Config,
}
