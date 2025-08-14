use base64::{engine::general_purpose::STANDARD, Engine};
use pasetors::{
    keys::{Generate, SymmetricKey},
    version4::V4,
};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let key: SymmetricKey<V4> = SymmetricKey::generate().expect("Cannot generate key");
        let encoded_key = STANDARD.encode(key.as_bytes());

        let default_key = Query::insert()
            .into_table(Config::Table)
            .columns([Config::Key, Config::Value])
            .values_panic(["auth_key".into(), encoded_key.into()])
            .to_owned();

        manager.exec_stmt(default_key).await?;

        // TODO: model config

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let delete_config = Query::delete().from_table(Config::Table).to_owned();
        manager.exec_stmt(delete_config).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Config {
    Table,
    Key,
    Value,
}
