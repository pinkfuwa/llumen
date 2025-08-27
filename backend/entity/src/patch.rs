use sea_orm::{DeriveActiveEnum, entity::prelude::*};

/// need patch `Message::Kind`
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MessageKind {
    User = 0,
    Assistant = 1,
    Reasoning = 2,
}
