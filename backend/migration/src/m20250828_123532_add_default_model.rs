use sea_orm_migration::{prelude::*, schema::*};

static DEFAULT_MODEL_CONFIG: &str =
    "openrouter_id=\"openai/gpt-oss-20b:free\"\ndisplay_name=\"GPT-OSS 20B\"";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let default_model = Query::insert()
            .into_table(Model::Table)
            .columns([Model::Config])
            .values_panic([DEFAULT_MODEL_CONFIG.into()])
            .to_owned();
        manager.exec_stmt(default_model).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Model {
    Table,
    Config,
}
