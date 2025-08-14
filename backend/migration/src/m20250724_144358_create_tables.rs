use pasetors::keys::Generate;
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

// password hash of P@88w0rd
static PASSWORD_HASH_ENCODE: &str =
    "argon2id$v=19$m=16,t=2,p=1$aTg5eTNyMmRzLTA$FM4qzh9B/+DdCVOiQQruGw";

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
            .values_panic(["admin".into(), PASSWORD_HASH_ENCODE.into()])
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
                    .col(string(Chat::Title))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Message::Table)
                    .col(pk_auto(Message::Id))
                    .col(integer(Message::ChatId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-message-chat_id-chat")
                            .from(Message::Table, Message::ChatId)
                            .to(Chat::Table, Chat::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(Message::Text))
                    .col(integer(Message::Kind))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-message-room_id")
                    .table(Message::Table)
                    .col(Message::ChatId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(Model::Table)
                    .col(pk_auto(Model::Id))
                    .col(string(Model::Config))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .if_not_exists()
                    .table(File::Table)
                    .col(pk_auto(File::Id))
                    .col(integer(File::MessageId))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file-message_id-message")
                            .from(File::Table, File::MessageId)
                            .to(Message::Table, Message::Id)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .col(string(File::Name))
                    .col(binary(File::Bytes))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx-file-message_id")
                    .table(File::Table)
                    .col(File::MessageId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Config::Table)
                    .col(string(Config::Key).primary_key())
                    .col(binary(Config::Value))
                    .to_owned(),
            )
            .await?;

        let key = pasetors::keys::SymmetricKey::generate().expect("Cannot generate");
        let insert_key = Query::insert()
            .into_table(Config::Table)
            .columns([Config::Key, Config::Value])
            .values_panic(["paseto_key".into(), key.as_bytes().into()])
            .to_owned();
        manager.exec_stmt(insert_key).await?;

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
            .drop_table(Table::drop().table(Message::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Model::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Config::Table).to_owned())
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
    Id,
    ModelId,
    OwnerId,
    Title,
}

#[derive(DeriveIden)]
enum Message {
    Table,
    Id,
    ChatId,
    Text,

    Kind,
}

#[derive(DeriveIden)]
enum Model {
    Table,
    Id,
    Config,
}

#[derive(DeriveIden)]
enum File {
    Table,
    Id,
    MessageId,
    Name,
    Bytes,
}

#[derive(DeriveIden)]
enum Config {
    Table,
    Key,
    Value,
}
