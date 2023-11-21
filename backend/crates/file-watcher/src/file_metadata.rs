use std::path::Path;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use config::CONFIG;
use entity::{file_metadata, files};
use file_format::FileFormat;
use sea_orm::{prelude::*, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::instrument;

use crate::{helpers::date::parse_db_date, FileWatcher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub file_id: i32,
    #[serde(rename = "type")]
    pub file_type: Option<String>,
    pub size: Option<i64>,
    pub created: Option<DateTime<Utc>>,
    pub modified: Option<DateTime<Utc>>,
}

impl From<file_metadata::Model> for FileMetadata {
    fn from(meta: file_metadata::Model) -> Self {
        Self {
            file_id: meta.file_id,
            file_type: meta.file_type,
            size: meta.file_size,
            created: meta.file_ctime.and_then(|x| parse_db_date(&x).ok()),
            modified: meta.file_mtime.and_then(|x| parse_db_date(&x).ok()),
        }
    }
}

impl FileWatcher {
    #[instrument(skip(self))]
    pub async fn get_or_create_file_metadata(&self, ulid: &str) -> Result<FileMetadata> {
        let txn = self.db().begin().await?;

        let (db_file, db_file_metadata) = files::Entity::find()
            .filter(files::Column::Ulid.eq(ulid))
            .find_also_related(file_metadata::Entity)
            .one(&txn)
            .await?
            .ok_or_else(|| anyhow!("File not found in database: {}", ulid))?;

        if let Some(db_file_metadata) = db_file_metadata {
            logger::trace!("File metadata exists in database");
            return Ok(db_file_metadata.into());
        }

        logger::debug!("Missing file metadata. Generating...");

        let file_path = CONFIG.app.directory.join(db_file.path);

        let file_type = match infer_file_type(&file_path) {
            Ok(x) => Some(x),
            Err(e) => {
                logger::warn!(err = ?e, "Failed to infer file type");
                None
            }
        };

        let meta = fs::metadata(file_path)
            .await
            .map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;

        let file_size: Option<i64> = meta.len().try_into().ok();
        let file_ctime = meta.created().ok().map(DateTime::<Utc>::from);
        let file_mtime = meta.modified().ok().map(DateTime::<Utc>::from);

        let mut file_meta = file_metadata::ActiveModel {
            file_id: Set(db_file.id),
            file_type: Set(file_type),
            file_size: Set(file_size),
            file_ctime: Set(file_ctime.map(|x| x.to_rfc3339())),
            file_mtime: Set(file_mtime.map(|x| x.to_rfc3339())),
            ..Default::default()
        };
        logger::trace!(meta = ?file_meta, "Adding file metadata to database");

        let file_meta = if let Some(f) = file_metadata::Entity::find()
            .filter(file_metadata::Column::FileId.eq(db_file.id))
            .one(&txn)
            .await?
        {
            file_meta.id = Set(f.id);
            file_meta.update(&txn).await?
        } else {
            file_meta.insert(&txn).await?
        };

        logger::trace!("Added meta");

        txn.commit().await?;

        Ok(file_meta.into())
    }
}

fn infer_file_type(file: &Path) -> Result<String> {
    infer::get_from_path(file)?
        .map(|x| x.mime_type().to_string())
        .or_else(|| {
            FileFormat::from_file(file)
                .map(|x| x.media_type().to_string())
                .ok()
        })
        .or_else(|| tree_magic_mini::from_filepath(file).map(ToString::to_string))
        .ok_or_else(|| anyhow!("Could not infer file type for file: {:?}", &file))
}
