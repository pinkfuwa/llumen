use std::sync::Arc;
use std::time::Duration;

use entity::file::{Column, Entity as File};
use sea_orm::{ColumnTrait, DbConn, EntityTrait, QueryFilter};
use time::OffsetDateTime;
use tokio::time::interval;

use super::blob::BlobDB;

pub struct FileCleanupService {
    conn: DbConn,
    blob: Arc<BlobDB>,
}

impl FileCleanupService {
    pub fn new(conn: DbConn, blob: Arc<BlobDB>) -> Self {
        Self { conn, blob }
    }

    pub fn start(self) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300));

            loop {
                interval.tick().await;

                if let Err(e) = self.cleanup_expired_files().await {
                    log::error!("Error during file cleanup: {:?}", e);
                }
            }
        });
    }

    async fn cleanup_expired_files(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = OffsetDateTime::now_utc().unix_timestamp() as i32;

        let expired_files = File::find()
            .filter(Column::ChatId.is_null())
            .filter(Column::ValidUntil.is_not_null())
            .filter(Column::ValidUntil.lte(now))
            .all(&self.conn)
            .await?;

        for file in expired_files {
            log::info!("Cleaning up expired file: {}", file.id);

            if let Err(e) = self.blob.delete(file.id) {
                log::warn!(
                    "Failed to delete blob for file {} (may not exist in redb): {:?}",
                    file.id,
                    e
                );
            }

            if let Err(e) = File::delete_by_id(file.id).exec(&self.conn).await {
                log::error!("Failed to delete file record {}: {:?}", file.id, e);
            }
        }

        Ok(())
    }
}
