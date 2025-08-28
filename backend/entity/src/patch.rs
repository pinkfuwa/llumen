use sea_orm::{DeriveActiveEnum, FromJsonQueryResult, entity::prelude::*};
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

/// need patch `Message::Kind`
#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
pub enum MessageKind {
    User = 0,
    Assistant = 1,
    Reasoning = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[typeshare]
pub struct UserPreference {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submit_on_enter: Option<String>,
}
