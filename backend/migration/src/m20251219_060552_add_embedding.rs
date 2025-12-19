use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Embedding::Table)
                    .if_not_exists()
                    .col(pk_auto(Embedding::Id))
                    .col(string(Embedding::ModelId))
                    .col(blob_null(Embedding::Sample))
                    .col(integer(Embedding::Count))
                    .col(integer(Embedding::Dimension).default(0))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Embedding::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Embedding {
    Table,
    Id,
    ModelId,
    Sample,
    Count,
    Dimension,
}
