use sea_orm_migration::{prelude::*, schema::*};

use super::entity::*;

// IMPORTANT NOTE (2026-02-13):
// Generated images (from image generation models) have owner_id = NULL and
// chat_id set. File access control has been updated to allow:
// 1. Files with owner_id matching the requesting user
// 2. Files with owner_id = NULL but chat_id belonging to the requesting user
//
// This prevents orphaned images in redb while allowing proper access to
// generated images. See: backend/src/routes/file/download.rs and
// backend/src/routes/file/image.rs
//
// Garbage collection (backend/src/utils/file_cleanup.rs) only removes files
// with:
// - chat_id IS NULL (temporary/orphaned files)
// - valid_until expired
// Therefore, generated images with chat_id set are protected from cleanup.
//
// NOTE: SQLite does NOT support DROP COLUMN, so owner_id column remains in
// schema even though it's nullable for generated images.

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(File::Table)
                    .if_not_exists()
                    .col(pk_auto(File::Id))
                    .col(integer_null(File::ChatId))
                    .col(integer_null(File::OwnerId))
                    .col(string_null(File::MimeType))
                    // File table store reference to redb
                    // It's set null, considering following case:
                    // 1. user got deleted
                    // 2. file got deleted
                    // 3. content in redb was never removed
                    //
                    // IMPORTANT: DO NOT change it to cascade
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file-chat_id-chat")
                            .from(File::Table, File::ChatId)
                            .to(Chat::Table, Chat::Id)
                            .on_update(ForeignKeyAction::SetNull)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-file-owner_id-user")
                            .from(File::Table, File::OwnerId)
                            .to(User::Table, User::Id)
                            .on_update(ForeignKeyAction::SetNull)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx-file-chat_id")
                    .table(File::Table)
                    .col(File::ChatId)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx-file-chat_id")
                    .table(File::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(File::Table).to_owned())
            .await?;

        Ok(())
    }
}
