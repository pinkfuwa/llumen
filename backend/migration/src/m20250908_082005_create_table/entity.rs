use sea_orm_migration::prelude::*;

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
    ChatId,
    MineType,
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
    Mode,
}

#[derive(DeriveIden)]
pub enum Message {
    Table,
    Id,
    ChatId,
    Kind,
    Price,
    TokenCount,
}

#[derive(DeriveIden)]
pub enum Chunk {
    Table,
    Id,
    MessageId,
    Content,
    Kind,
}
