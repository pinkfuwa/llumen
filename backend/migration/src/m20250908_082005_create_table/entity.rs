use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveIden)]
pub enum User {
    Table,
    Id,
    Name,
    Password,
    Preference,
}

#[derive(DeriveIden)]
pub enum Model {
    Table,
    Id,
    Config,
}

#[derive(DeriveIden)]
pub enum File {
    Table,
    Id,
    MessageId,
    Name,
}

#[derive(DeriveIden)]
pub enum Config {
    Table,
    Key,
    Value,
}

#[derive(DeriveIden)]
pub enum Tool {
    Table,
    ChatId,
    FunctionName,
    State,
}

#[derive(DeriveIden)]
pub enum Chat {
    Table,
    Id,
    ModelId,
    OwnerId,
    Title,
}

#[derive(DeriveIden)]
pub enum Message {
    Table,
    Id,
    ChatId,
    Kind,
}

#[derive(DeriveIden)]
pub enum Chunk {
    Table,
    Id,
    MessageId,
    Content,
    Kind,
}
